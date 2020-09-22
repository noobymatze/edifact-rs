use std::io;
use combine::easy;
use combine::stream::position::SourcePosition;
use core::fmt;
use std::io::Read;
use crate::mig::description;
use crate::mig::error::InterchangeError;

pub mod value;
mod parser;

// type ParseError = easy::Errors<char, String, SourcePosition>;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Parse(easy::Errors<char, String, SourcePosition>),
    Mig(InterchangeError)
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(error) => error.fmt(f),
            Error::Parse(error) => error.fmt(f),
            Error::Mig(_) => Ok(())
        }
    }
}

impl From<InterchangeError> for Error {
    fn from(e: InterchangeError) -> Self {
        Error::Mig(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<easy::Errors<char, String, SourcePosition>> for Error {
    fn from(e: easy::Errors<char, String, SourcePosition>) -> Self {
        Error::Parse(e)
    }
}


pub fn decode<R: Read>(known: Vec<description::Interchange>, input: &mut R) -> Result<value::Interchange, Error> {
    let interchange = parser::parse(input)?;
    let result = value::match_interchange(&known[0], interchange)?;
    Ok(result)
}
