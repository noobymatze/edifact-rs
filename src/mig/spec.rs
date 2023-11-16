//! Parse edi@energy MIG PDF files
//!
//! This module implements a Parser for any Message Integration Guide
//! (MIG) specified by [edi@energy](http://www.edi-energy.de/). The
//! pdf documents will be converted by the command line program `pdftotext`
//! internally, so it has to be available on the path.
//!
//! ## HIGH-LEVEL DESCRIPTION
//!
//! This section will introduce the high-level description to parsing
//! MIGs. A MIG is a structured specification of an EDIFACT message or an
//! interchange. It consists of the following sections.
//!
//! 1. Title page
//! 2. Table of contents
//! 3. Message structure
//! 4. Message diagram
//! 5. Segment layouts
//! 6. Change log
//!
//! For the purpose of parsing the specification from this document, *the
//! table of contents*, *the message diagram* and the *change log* can be
//! neglected. Parsing the title page and the message structure can be
//! used to augument the generated code for specifying the respective
//! message or the resulting types in other programming languages, but is
//! not strictly necessary. Parsing the segment layouts is paramount
//! though, thus being the most important task of this undertaking.
//!
//! As such, the document will parse *title page*, *message structure* and
//! *segment layouts* and ignore the rest. Parsing happens twofold. First,
//! the aforementioned sections will be parsed, splitting up the segment
//! layouts into distinct parts. This is to isolate them and allow further
//! analysis individually. Furthermore, it allows for better reasoning
//! about an error case, thus mitigating the non-descriptive error
//! handling by attoparsec, as well as catching an error for every
//! segment, instead of just for the first, if doing one pass.
use std::ops::{Index, Range, RangeFrom, RangeTo};
use std::path::Path;
use std::process;

use nom::branch::alt;
use nom::bytes::complete::{tag, take_until, take_while, take_while1};
use nom::character::complete::{
    line_ending, multispace0, newline, not_line_ending, space0,
};
use nom::combinator::{map, map_res};
use nom::error::{convert_error, ContextError, ParseError, VerboseError};
use nom::multi::many_till;
use nom::sequence::{delimited, tuple};
use nom::{
    AsChar, Compare, Finish, IResult, InputIter, InputLength, InputTake,
    InputTakeAtPosition, Slice,
};

use crate::mig::description as desc;

#[derive(Debug)]
pub enum Error {
    PdfToText(String),
    PathCannotBeConvertedToStr(),
    CouldNotReadTxtFile(std::io::Error),
}

/// Parses the given [path] into a [desc::Interchange].
pub fn parse<P: AsRef<Path>>(path: P) -> Result<desc::Interchange, Error> {
    let file =
        path.as_ref().to_str().ok_or(Error::PathCannotBeConvertedToStr())?;
    if file.ends_with(".txt") {
        let content = std::fs::read_to_string(file)
            .map_err(Error::CouldNotReadTxtFile)?;
        parse_string(content)
    } else {
        let output = process::Command::new("pdftotext")
            .arg("-layout")
            .arg(file)
            .arg("-")
            .output()
            .expect("Failed to execute pdftotext");

        if !output.status.success() {
            let error = String::from_utf8(output.stderr).expect("Works");
            Err(Error::PdfToText(error))
        } else {
            let input = String::from_utf8(output.stdout).expect("Works");
            parse_string(input)
        }
    }
}

type ParseResult<I, O, E = VerboseError<I>> = IResult<I, O, E>;

pub fn parse_string(input: String) -> Result<desc::Interchange, Error> {
    let result = mig(input.as_str());
    match result {
        Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
            println!("Error:\n{}", convert_error(input.as_str(), e));
        }
        Err(error) => println!("Error happened"),
        Ok((_, wohoo)) => {
            println!("{}", wohoo)
        }
    }

    Err(Error::PdfToText("Wooohooo".to_string()))
}

fn mig(input: &str) -> ParseResult<&str, String> {
    map(
        tuple((
            many_till(line, start_of_toc),
            many_till(line, start_of_message_structure),
            message_structure,
        )),
        |(_, _, structure)| structure,
    )(input)
}

fn line(input: &str) -> ParseResult<&str, &str> {
    map(tuple((take_while(|c: char| c != '\n'), line_ending)), |(r, _)| r)(
        input,
    )
}

fn start_of<T, I>(section_name: T) -> impl FnMut(I) -> ParseResult<I, ()>
where
    I: InputTake + Compare<T>,
    I: Slice<Range<usize>> + Slice<RangeFrom<usize>> + Slice<RangeTo<usize>>,
    I: InputIter + InputLength,
    I: Compare<&'static str>,
    I: InputTakeAtPosition,
    I: Clone,
    <I as InputTakeAtPosition>::Item: AsChar + Clone,
    T: InputLength + Clone,
{
    map(tuple((multispace0, tag(section_name), line_ending)), |_| ())
}

fn start_of_toc(input: &str) -> ParseResult<&str, ()> {
    map(
        tuple((
            multispace0,
            tag("Nachrichtenstruktur"),
            space0,
            take_while1(|c| c == '.'),
            space0,
            take_while1(|c: char| c.is_ascii_digit()),
            line_ending,
        )),
        |_| (),
    )(input)
}

fn start_of_message_structure(input: &str) -> ParseResult<&str, ()> {
    start_of("Nachrichtenstruktur")(input)
}

fn start_of_segment_layout(input: &str) -> ParseResult<&str, ()> {
    start_of("Segmentlayout")(input)
}

fn start_of_diagram(input: &str) -> ParseResult<&str, ()> {
    start_of("Diagramm")(input)
}

fn start_of_changelog(input: &str) -> ParseResult<&str, ()> {
    start_of("Änderungshistorie")(input)
}

/// Parsing the message structure is straight forward. The only caveat to
/// consider is to filter out the resulting lines, which are part of the
/// footer.
fn message_structure(input: &str) -> ParseResult<&str, String> {
    let is_relevant = |l: &str| {
        !is_part_of_message_structure_header(l)
            && !is_part_of_footer(l)
            && !l.starts_with("BDEW")
            && l != ""
    };

    let (input, (lines, _)) = many_till(
        delimited(space0, not_line_ending, line_ending),
        start_of_diagram,
    )(input)?;

    let filtered_lines = lines
        .iter()
        .map(|l| l.trim_start().to_string())
        .filter(|l| is_relevant(l))
        .collect::<Vec<_>>();

    let mut result = String::new();
    for line in filtered_lines {
        if line.starts_with("0") {
            result.push('\n');
            result.push_str(line.as_str());
        } else {
            result.push(' ');
            result.push_str(line.as_str());
        }
    }

    Ok((input, result))
}

fn is_part_of_message_structure_header(line: &str) -> bool {
    let line = line.trim_start();
    line.starts_with("Nachrichtenstruktur")
        || line.starts_with("Zähler")
        || (line.starts_with("Status") && line.contains("MaxWdh"))
}

fn is_part_of_footer(line: &str) -> bool {
    let trimmed_line = line.trim();
    trimmed_line.starts_with("Zähler =")
        || trimmed_line.starts_with("Bez =")
        || trimmed_line.starts_with("Nr =")
        || trimmed_line.starts_with("MaxWdh =")
        || trimmed_line.starts_with("EDI@Energy")
        || trimmed_line.starts_with("Version")
        || trimmed_line.starts_with("UTILMD-")
        || trimmed_line == "Strom"
}

// SEGMENTS

fn end_of_segment_layout(input: &str) -> ParseResult<&str, ()> {
    map(alt((start_of_segment_layout, end_of_segment_layout)), |_| ())(input)
}

//fn start_of_elements(input: &str) -> ParseResult<&str, ()> {}

fn standard_bdew_line(input: &str) -> ParseResult<&str, ()> {
    map(
        tuple((
            tuple((space0, tag("Standard"))),
            tuple((space0, tag("BDEW"))),
        )),
        |_| (),
    )(input)
}

fn segment_column_headers(input: &str) -> ParseResult<&str, ()> {
    map(
        tuple((
            tuple((space0, tag("Zähler"))),
            tuple((space0, tag("Nr"))),
            tuple((space0, tag("Bez"))),
            tuple((space0, tag("St"))),
            tuple((space0, tag("MaxWdh"))),
            tuple((space0, tag("St"))),
            tuple((space0, tag("MaxWdh"))),
            tuple((space0, tag("Ebene"))),
            tuple((space0, tag("Name"))),
        )),
        |_| (),
    )(input)
}

#[cfg(test)]
mod tests {
    use crate::mig::spec::start_of;

    #[test]
    fn test_start_of() {
        assert_eq!(start_of("[my_section]")(" [my_section]\n"), Ok(("", ())));
    }
}
