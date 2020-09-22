//! This module provides types and parsers to parse edifact messages.
use std::fmt;

use combine::{
    any, attempt, eof, Parser, position, RangeStream, sep_by, sep_by1, Stream,
};
use combine::error::ParseError;
use combine::parser::char::{char, spaces, string};
use combine::parser::combinator::recognize;
use combine::parser::range::take_while1;
use combine::parser::repeat::{escaped, repeat_until};
use combine::parser::token::value;
use combine::stream::position::SourcePosition;
use combine::stream::Range;
use serde::{Deserialize, Serialize};

use crate::mig::either::Either;

/// The UNA string advice is a service segment, which declares separators and
/// special characters, such as escaping characters, in any EDIFACT message.
/// It is optional and always the first segment in any interchange.
///
/// # Example
///
/// Here is an example: UNA:+.? '
#[derive(Clone, Debug, Serialize, Deserialize, Copy)]
pub struct UNA {
    pub component_sep: char,
    pub element_sep: char,
    pub decimal_char: char,
    pub escape: char,
    pub reserved: char,
    pub segment_sep: char,
}

impl UNA {
    /// Create a UNA service segment with default separators.
    ///
    /// These are:
    ///
    ///   - `:` as a component separator
    ///   - `+` as an element separator
    ///   - `.` as decimal separator
    ///   - `?` as escaping symbol
    ///   - `'` as segment separator
    ///
    pub fn default() -> UNA {
        UNA {
            component_sep: ':',
            element_sep: '+',
            decimal_char: '.',
            escape: '?',
            reserved: ' ',
            segment_sep: '\'',
        }
    }

    /// Create a new UNA service segment with the specified separators.
    pub fn new(
        component_sep: char,
        element_sep: char,
        decimal_char: char,
        escape: char,
        reserved: char,
        segment_sep: char,
    ) -> UNA {
        UNA {
            component_sep: component_sep,
            element_sep: element_sep,
            decimal_char: decimal_char,
            escape: escape,
            reserved: reserved,
            segment_sep: segment_sep,
        }
    }

    /// Check, if the given character is a component, element or segment separator.
    ///
    /// # Examples
    ///
    /// ```
    /// let x: bool = UNA::default().is_separator(':')
    /// assert_eq!(x, true)
    /// ```
    fn is_separator(&self, c: char) -> bool {
        self.component_sep == c
            || self.segment_sep == c
            || self.element_sep == c
    }

    /// Check, if the given character is the escape symbol.
    ///
    /// # Examples
    ///
    /// ```
    /// let x: bool = UNA::default().is_escape('?')
    /// assert_eq!(x, true)
    /// ```
    fn is_escape(&self, c: char) -> bool {
        self.escape == c
    }

    fn parser<Input>() -> impl Parser<Input, Output = Self>
        where
            Input: Stream<Token = char>,
            Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
    {
        let p = (string("UNA"), any(), any(), any(), any(), any(), any()).map(
            |(_, csep, esep, dec, esc, res, ssep)| {
                UNA::new(csep, esep, dec, esc, res, ssep)
            },
        );

        let una = UNA::default();
        attempt(p).or(value(una))
    }
}

impl fmt::Display for UNA {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "UNA{}{}{}{}{}{}",
            self.component_sep,
            self.element_sep,
            self.decimal_char,
            self.escape,
            self.reserved,
            self.segment_sep
        )
    }
}

/// An `Interchange` consists of a list of segments and a starting
/// UNA service segment. Technically, this is not true, since the
/// interchange also
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Interchange {
    pub una: UNA,
    pub segments: Vec<Segment>,
}

impl Interchange {
    /// Create a parser for parsing an Interchange.
    ///
    /// The parser is designed to mostly succeed, which is why the Interchange,
    /// which inherently has more structure than being a list of segments,
    /// does not have more structure.
    pub fn parser<Input>() -> impl Parser<Input, Output = Interchange>
        where
            Input: RangeStream<Token = char, Position = SourcePosition>,
            Input::Range: Range,
            Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
    {
        UNA::parser().then(|una| {
            repeat_until(attempt(Segment::parser(&una)), eof()).map(
                move |segments| Interchange { una: una, segments: segments },
            )
        })
    }
}

/// A `Segment` represents a segment, which always starts with a
/// data element, called a `tag` and a number of follow-up
/// elements.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Segment {
    pub tag: DataElement,
    pub elements: Vec<Either<Composite, DataElement>>,
}

impl Segment {
    pub fn parser<Input>(una: &UNA) -> impl Parser<Input, Output = Segment>
        where
            Input: RangeStream<Token = char, Position = SourcePosition>,
            Input::Range: Range,
            Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
    {
        let element = attempt(Composite::parser(una).map(|x| Either::Left(x)))
            .or(DataElement::parser(una.clone()).map(|x| Either::Right(x)));

        (
            DataElement::parser(una.clone()),
            char(una.clone().element_sep),
            sep_by(element, char(una.clone().element_sep)),
            char(una.clone().segment_sep),
            attempt(spaces()),
        )
            .map(|(tag, _, elements, _, _)| Segment {
                tag,
                elements,
            })
    }
}

/// A `Composite` represents a composite element as part of
/// a segment. It has to have at least two elements to be
/// categorized as such.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Composite {
    pub elements: Vec<DataElement>,
}

impl Composite {
    pub fn parser<Input>(una: &UNA) -> impl Parser<Input, Output = Composite>
        where
            Input: RangeStream<Token = char, Position = SourcePosition>,
            Input::Range: Range,
            Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
    {
        (
            DataElement::parser(*una),
            char(una.clone().component_sep),
            sep_by1(
                DataElement::parser(*una),
                char(una.component_sep.clone()),
            ),
        )
            .map(
                |(first, _, rest): (DataElement, char, Vec<DataElement>)| {
                    let mut elements = rest;
                    elements.insert(0, first);
                    Composite { elements }
                },
            )
    }
}

/// A `DataElement` represents a single data element and its start and
/// end position inside a composite element or segment.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataElement {
    pub start: Position,
    pub end: Position,
    /// The parsed value. Escaped characters will be contained without
    /// the escaping character. Decimal strings will be normalized to
    /// use '.'.
    pub value: String,
}

impl DataElement {
    pub fn parser<Input>(una: UNA) -> impl Parser<Input, Output = DataElement>
        where
            Input: RangeStream<Token = char, Position = SourcePosition>,
            Input::Range: Range,
            Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
    {
        let text = recognize(escaped(
            take_while1(move |c| !una.is_escape(c) && !una.is_separator(c)),
            una.escape,
            any(),
        ));
        (position(), text, position()).map(
            |(start, value, end): (SourcePosition, String, SourcePosition)| {
                DataElement {
                    start: Position { line: start.line, column: start.column },
                    end: Position { line: end.line, column: end.column },
                    value,
                }
            },
        )
    }
}

/// A `Position` is isomorphic to a `SourcePosition` and used to track
/// the position of a data element in the input stream. It is used,
/// to be able to implement serialize and deserialize traits.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Position {
    pub line: i32,
    pub column: i32,
}

