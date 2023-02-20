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
use std::process::ExitStatus;

use nom::bytes::complete::{tag, take_while, take_while1};
use nom::character::complete::{line_ending, multispace0, newline, space0};
use nom::combinator::{map, map_res};
use nom::error::{convert_error, ContextError, ParseError, VerboseError};
use nom::multi::many_till;
use nom::sequence::tuple;
use nom::{
    AsChar, Compare, Finish, IResult, InputIter, InputLength, InputTake,
    InputTakeAtPosition, Slice,
};

use crate::mig::description as desc;

#[derive(Debug)]
pub enum Error {
    PdfToText(String),
}

/// Parses the given [path] into a [desc::Interchange].
pub fn parse<P: AsRef<Path>>(path: P) -> Result<desc::Interchange, Error> {
    let file = path.as_ref().to_str().expect("Expect this to work.");
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

type ParseResult<I, O, E = VerboseError<I>> = IResult<I, O, E>;

pub fn parse_string(input: String) -> Result<desc::Interchange, Error> {
    let result = mig(input.as_str());
    match result {
        Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
            println!("Error:\n{}", convert_error(input.as_str(), e));
        }
        Err(error) => println!("Error happened"),
        Ok((_, wohoo)) => {
            for (i, line) in wohoo.iter().enumerate() {
                println!("{}: {}", i, line)
            }
        }
    }

    Err(Error::PdfToText("Wooohooo".to_string()))
}

fn mig(input: &str) -> ParseResult<&str, Vec<&str>> {
    map(many_till(line, start_of_toc), |(lines, _)| lines)(input)
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

#[cfg(test)]
mod tests {
    use crate::mig::spec::start_of;

    #[test]
    fn test_start_of() {
        assert_eq!(start_of("[my_section]")(" [my_section]\n"), Ok(("", ())));
    }
}
