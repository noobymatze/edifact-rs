use crate::mig::description as desc;
use crate::mig::description::{Format, Size, St, Usage};
use crate::mig::either::Either;
use crate::mig::error::{CompositeError, DataElementError, SegmentError, SyntaxError, InterchangeError, MessageError};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use crate::mig::decode::parser;
use crate::mig::decode::parser::value;

#[derive(Debug, Serialize, Deserialize)]
pub struct Interchange {
    pub segments: Vec<Either<Segmentgroup, Segment>>,
    //unb: Segment,
    //messages: Vec<Message>,
    //unz: Segment,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    unh: Segment,
    segments: Vec<Either<Segmentgroup, Segment>>,
    unt: Segment,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Segmentgroup {
    counter: String,
    label: String,
    st: desc::St,
    max_reps: u64,
    level: u64,
    name: String,
    comment: Option<String>,
    segments: Vec<Either<Segmentgroup, Segment>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Segment {
    index: usize,
    counter: String,
    number: u64,
    tag: String,
    st: desc::St,
    max_reps: u64,
    level: u64,
    name: String,
    comment: Option<String>,
    elements: Vec<Either<Composite, DataElement>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Composite {
    index: usize,
    label: String,
    name: String,
    st: desc::St,
    elements: Vec<DataElement>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataElement {
    description: desc::DataElement,
    index: usize,
    value: Option<Matched>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Matched {
    Text(String),
    Int(u64),
    Decimal(f64),
}

// MATCHING

pub fn match_interchange(desc: &desc::Interchange, value: parser::value::Interchange) -> Result<Interchange, InterchangeError> {
    let mut segments = vec![
        Either::Right(desc.unb.clone()),
        Either::Right(desc.message.unh.clone()),
    ];
    let mut s = desc.message.segments.clone();
    segments.append(&mut s);
    segments.append(&mut vec![
        Either::Right(desc.message.unt.clone()),
        Either::Right(desc.unz.clone()),
    ]);
    let mut values = value.segments;
    values.reverse();
    match matching(0, &segments, &mut values) {
        (_, Ok(result)) => {
            Ok(Interchange { segments: result })
        }
        (_, Err(error)) => {
            let msg_error = MessageError {
                pos: 0,
                service_segment_error: None,
                segment_errors: error,
            };

            Err(InterchangeError {
                pos: 0,
                message_errors: vec![msg_error],
                service_segment_error: None
            })
        }
    }
}

fn matching(
    pos: usize,
    descs: &Vec<Either<desc::Segmentgroup, desc::Segment>>,
    stack: &mut Vec<parser::value::Segment>,
) -> (usize, Result<Vec<Either<Segmentgroup, Segment>>, Vec<SegmentError>>) {
    let mut index = pos;
    let mut matches: Vec<Either<Segmentgroup, Segment>> = vec![];
    let mut errors: Vec<SegmentError> = vec![];
    for (_counter, next) in &descs.iter().group_by(|v| get_counter(v)) {
        let mut next_descs: Vec<_> = next.collect();
        let check_qualifier = next_descs.len() > 1;
        while let Some(v) = stack.pop() {
            let next_match = next_descs.iter().position(|d| match d {
                Either::Left(desc) => matches_segmentgroup(desc, check_qualifier, &v),
                Either::Right(desc) => matches_segment(desc, check_qualifier, &v),
            });

            if let Some(i) = next_match {
                match &next_descs[i] {
                    Either::Right(desc) => {
                        match match_segment(index, desc, &v) {
                            Ok(matched) => {
                                matches.push(Either::Right(matched))
                            }
                            Err(error) => errors.push(error),
                        };
                        index += 1;
                        // TODO: Or if they have been consumed
                        if desc.max_reps == 1 {
                            next_descs.remove(i);
                        }
                    }
                    Either::Left(desc) => {
                        match matching(index, &desc.segments, stack) {
                            (next, Ok(values)) => {
                                matches.push(Either::Left(Segmentgroup {
                                    counter: desc.counter.clone(),
                                    label: desc.label.clone(),
                                    st: desc.st,
                                    max_reps: desc.max_reps,
                                    level: desc.level,
                                    name: desc.name.clone(),
                                    comment: desc.comment.clone(),
                                    segments: values,
                                }));
                                index += next;
                            }
                            (next, Err(mut error)) => {
                                index += next;
                                errors.append(&mut error)
                            }
                        }
                    }
                }
            } else {
                // Push the consumed value back onto the stack
                stack.push(v);
                break;
            }
        }
    }

    if !errors.is_empty() {
        (index, Err(errors))
    } else {
        (index, Ok(matches))
    }
}

/// Returns, if this segmentgroup starts with the given value.
fn matches_segmentgroup(
    desc: &desc::Segmentgroup,
    check_qualifier: bool,
    value: &value::Segment,
) -> bool {
    match desc.segments.as_slice() {
        [Either::Right(segment), ..] => {
            matches_segment(segment, check_qualifier, &value)
        }
        _ => false,
    }
}

/// Returns, whether the given value matches this segment description.
pub fn matches_segment(
    desc: &desc::Segment,
    check_qualifier: bool,
    value: &value::Segment,
) -> bool {
    if !check_qualifier {
        return desc.tag == value.tag.value;
    } else if desc.tag != value.tag.value {
        return false;
    }

    let qualifier = desc
        .elements
        .get(0)
        .and_then(|element| match element {
            Either::Left(composite) => composite.elements.get(0),
            Either::Right(data_element) => Some(data_element),
        })
        .and_then(|data_element| {
            if data_element.is_qualifier() {
                Some(data_element.usage.clone())
            } else {
                None
            }
        });

    let option_data_element =
        value.elements.get(0).and_then(|element| match element {
            Either::Left(composite) => composite.elements.get(0),
            Either::Right(data_element) => Some(data_element),
        });

    match (qualifier, option_data_element) {
        (
            Some(Usage::OneOf { choices, comment: _ }),
            Some(data_element),
        ) => choices.iter().any(|c| c.value == data_element.value),
        (
            Some(Usage::Static { value, comment: _ }),
            Some(data_element),
        ) => value.value == data_element.value,
        _ => false,
    }
}

fn get_counter(desc: &Either<desc::Segmentgroup, desc::Segment>) -> String {
    match desc {
        Either::Left(v) => v.counter.clone(),
        Either::Right(v) => v.counter.clone(),
    }
}

fn match_segment(
    pos: usize,
    desc: &desc::Segment,
    segment: &parser::value::Segment,
) -> Result<Segment, SegmentError> {
    let mut descs = desc.elements.iter();
    let mut values = segment.elements.iter();

    // Essentially, we are zipping descriptions and values here
    // This is done with a loop, since rust does not have TCO
    // STATE
    let mut position: usize = 0;
    let mut syntax_error: Option<SyntaxError> = None;
    let mut matches: Vec<Either<Composite, DataElement>> = vec![];
    let mut errors: Vec<Either<CompositeError, DataElementError>> = vec![];

    loop {
        match (descs.next(), values.next()) {
            // No descriptions and no values anymore, we are done
            (None, None) => break,
            (None, Some(_)) => {
                // Too many elements. edi@energy does not support repetition,
                // therefore no descriptions available anymore, bail
                syntax_error = Some(SyntaxError::too_many_parts());
                break;
            }
            (Some(Either::Right(desc)), None) => {
                // Found a description, but no corresponding value. This is
                // fine, if the element is not required.
                if desc.st.is_required() {
                    errors.push(Either::Right(DataElementError::new(
                        position,
                        SyntaxError::missing(),
                    )));
                }
            }
            (Some(Either::Left(desc)), None) => {
                // Found a description, but no corresponding value. This is
                // fine, if the element is not required.
                if desc.st.is_required() {
                    errors.push(Either::Right(DataElementError::new(
                        position,
                        SyntaxError::missing(),
                    )));
                }
            }
            (Some(Either::Right(_)), Some(Either::Left(_))) => {
                // Assumption: Every composite with only one element is
                // a  data element. Now: Expecting a data element, but
                // finding a composite is completely wrong. If it had only
                // one element, we could interpret it as a data element
                // making the whole thing more robust, but we skip that here
                errors.push(Either::Right(DataElementError::new(
                    position,
                    SyntaxError::invalid_value(),
                )))
            }
            (Some(Either::Left(desc)), Some(Either::Right(value))) => {
                // Found a composite description, but a data element value
                // this is only okay, if the composite has one element or
                // is not required and the value is empty
                if !(value.value == "" && desc.st == St::N) {
                    let composite_value =
                        value::Composite { elements: vec![value.clone()] };
                    match match_composite(position, desc, &composite_value) {
                        Ok(composite) => matches.push(Either::Left(composite)),
                        Err(error) => errors.push(Either::Left(error)),
                    }
                }
            }
            (Some(Either::Left(desc)), Some(Either::Left(value))) => {
                match match_composite(position, desc, value) {
                    Ok(composite) => matches.push(Either::Left(composite)),
                    Err(error) => errors.push(Either::Left(error)),
                }
            }
            (Some(Either::Right(desc)), Some(Either::Right(value))) => {
                // TODO: make data_element borrow
                match match_data_element(position, desc.clone(), value.clone())
                {
                    Ok(data_element) => {
                        matches.push(Either::Right(data_element))
                    }
                    Err(error) => errors.push(Either::Right(error)),
                }
            }
        }
        position += 1;
    }

    if !errors.is_empty() || syntax_error.is_some() {
        Err(SegmentError {
            pos: pos,
            syntax_error: syntax_error,
            errors: errors,
        })
    } else {
        Ok(Segment {
            index: pos,
            counter: desc.counter.clone(),
            number: desc.number,
            tag: desc.tag.clone(),
            st: desc.st,
            max_reps: desc.max_reps,
            level: desc.level,
            name: desc.name.clone(),
            comment: desc.comment.clone(),
            elements: matches,
        })
    }
}

fn match_composite(
    pos: usize,
    desc: &desc::Composite,
    composite: &parser::value::Composite,
) -> Result<Composite, CompositeError> {
    if desc.st.is_required() && composite.elements.is_empty() {
        Err(CompositeError::syntax_error(pos, SyntaxError::missing()))
    } else {
        let result =
            match_composite_help(pos, &desc.elements, &composite.elements);

        match result {
            Ok(matches) => Ok(Composite {
                index: pos,
                label: desc.label.clone(),
                st: desc.st,
                name: desc.name.clone(),
                elements: matches,
            }),
            Err(error) => Err(error),
        }
    }
}

fn match_composite_help(
    pos: usize,
    descs_vec: &Vec<desc::DataElement>,
    values_vec: &Vec<parser::value::DataElement>,
) -> Result<Vec<DataElement>, CompositeError> {
    let mut descs = descs_vec.iter();
    let mut values = values_vec.iter();

    // Essentially, we are zipping descriptions and values here
    // This is done with a loop, since rust does not have TCO
    // STATE
    let mut position: usize = 0;
    let mut syntax_error: Option<SyntaxError> = None;
    let mut matches: Vec<DataElement> = vec![];
    let mut errors: Vec<DataElementError> = vec![];

    // LOOP
    loop {
        match (descs.next(), values.next()) {
            // No descriptions and no values anymore, we are done
            (None, None) => break,
            (None, Some(_)) => {
                // Too many data elements. edi@energy does not support repetition,
                // therefore no descriptions available anymore, bail
                syntax_error = Some(SyntaxError::too_many_parts());
                break;
            }
            (Some(desc), None) => {
                // Found a description, but no corresponding value. This is
                // fine, if the data element is not required.
                if desc.st.is_required() {
                    errors.push(DataElementError::new(
                        position,
                        SyntaxError::missing(),
                    ))
                }
            }
            (Some(desc), Some(value)) => {
                match match_data_element(position, desc.clone(), value.clone())
                {
                    Ok(matched) => matches.push(matched),
                    Err(error) => errors.push(error),
                }
            }
        }
        position += 1;
    }

    if !errors.is_empty() || syntax_error.is_some() {
        Err(CompositeError {
            pos: pos,
            syntax_error: syntax_error,
            errors: errors,
        })
    } else {
        Ok(matches)
    }
}

fn match_data_element(
    pos: usize,
    desc: desc::DataElement,
    element: parser::value::DataElement,
) -> Result<DataElement, DataElementError> {
    let st_checked = check_st(desc.st, element.value)
        .map_err(|e| DataElementError::new(pos, e))?;

    if st_checked.is_empty() {
        Ok(DataElement { index: pos, description: desc, value: None })
    } else {
        let value =
            check_format(desc.st, desc.format, desc.length, st_checked)
                .map_err(|e| DataElementError::new(pos, e))?;

        Ok(DataElement {
            index: pos,
            description: desc,
            value: Some(Matched::Text(value)),
        })
    }
}

// CHECKING

fn check_st(st: St, input: String) -> Result<String, SyntaxError> {
    if input.is_empty() && st.is_required() {
        Err(SyntaxError::missing())
    } else if !input.is_empty() && st.is_not_used() {
        Err(SyntaxError::invalid_value())
    } else {
        Ok(input)
    }
}

fn check_format(
    st: St,
    format: Format,
    length: usize,
    input: String,
) -> Result<String, SyntaxError> {
    match format {
        Format::Alphanumeric(size) => check_size(st, size, length, input),
        Format::Alpha(size) => check_size(st, size, length, input),
        Format::Numeric(size) => check_size(st, size, length, input),
    }
}

fn check_size(
    st: St,
    size: Size,
    length: usize,
    input: String,
) -> Result<String, SyntaxError> {
    match size {
        Size::Exactly => {
            if (st.is_optional() || st.is_not_used()) && input == "" {
                Ok(input)
            } else if input.len() < length {
                Err(SyntaxError::data_element_too_short())
            } else if input.len() > length {
                Err(SyntaxError::data_element_too_long())
            } else {
                Ok(input)
            }
        }
        Size::AtMost => {
            if input.len() > length {
                Err(SyntaxError::data_element_too_long())
            } else {
                Ok(input)
            }
        }
    }
}
