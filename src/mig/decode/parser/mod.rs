pub mod value;

use combine::stream::position::Stream;
use combine::EasyParser;
use std::io::Read;
use crate::mig::decode::Error;


pub fn parse<R: Read>(input: &mut R) -> Result<value::Interchange, Error> {
    let mut contents = String::new();
    input.read_to_string(&mut contents)?;
    let i = &*contents;

    let mut parser = value::Interchange::parser();
    let (interchange, _) = parser
        .easy_parse(Stream::new(i))
        .map_err(|e| Error::Parse(e.map_range(|s| s.to_string())))?;

    Ok(interchange)
}
