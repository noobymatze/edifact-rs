//! This module contains data types, which represent the description of
//! EDIFACT messages in the subset edi@energy, which is used in the German
//! energy market.
//!
//! The specifications for the descriptions are published by the BDEW at
//! [www.edi-energy.de](https://www.edi-energy.de), usually in a 6 month
//! cycle.
//!
//! The problem with these specifications is, that they are not machine
//! readable. The data types in this module try to formalize them.

use crate::mig::either::Either;
use serde::de::{self, Deserializer, Visitor};
use serde::{Deserialize, Serialize};
use std::fmt;

/// An envelope around a set of messages.
///
/// An [Interchange] may start with a UNA, always followed by
/// a UNB segment and ends with a UNZ segment. In the German
/// energy market, interchanges are homogeneous, meaning, they
/// only contain messages of the same type.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Interchange {
    pub unb: Segment,
    pub message: Message,
    pub unz: Segment,
}

/// An envelope around a set of segments.
///
/// A [Message] always starts with a UNH segment and ends
/// with a UNT segment. The UNH segment identifies the kind
/// of message, e.g. APERAK, MSCONS, CONTRL or any other
/// kind of message.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub unh: Segment,
    pub segments: Vec<Either<Segmentgroup, Segment>>,
    pub unt: Segment,
}

/// A group of segments.
///
/// A `Segmentgroup` must consist of at least one segment.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Segmentgroup {
    pub counter: String,
    pub label: String,
    pub st: St,
    pub max_reps: u64,
    pub level: u64,
    pub name: String,
    pub comment: Option<String>,
    pub segments: Vec<Either<Segmentgroup, Segment>>,
}


/// A set of [Composite](struct.Composite.html) or
/// [DataElement](struct.DataElement.html) elements.
///
/// A `Segment` always starts with a data element consisting of
/// three captialized letters. This data element is called a tag
/// or segment tag. If a tag starts with a 'U', it means, that
/// the `Segment` is a service segment.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    pub counter: String,
    pub number: u64,
    pub tag: String,
    pub st: St,
    pub max_reps: u64,
    pub level: u64,
    pub name: String,
    pub comment: Option<String>,
    pub elements: Vec<Either<Composite, DataElement>>,
}


///
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Composite {
    pub label: String,
    pub name: String,
    pub st: St,
    pub elements: Vec<DataElement>,
}


/// A description, representing a data element as defined in a message integration guide.
///
///
/// # Example
///
/// The following example has been taken from the UCI segment in the [CONTRL](https://www.edi-energy.de/index.php?id=38&tx_bdew_bdew%5Buid%5D=697&tx_bdew_bdew%5Baction%5D=download&tx_bdew_bdew%5Bcontroller%5D=Dokument&cHash=27293a7bdcf9496c3e997789ba07d658) MIG.
///
/// ```
/// 0020 | Datenaustauschreferenz | M | an..14 | Eindeutige Referenz...
/// ^^^^   ^^^^^^^^^^^^^^^^^^^^^^   ^   ^^^^^^   ^^^^^^^^^^^^^^^^^^^^^^
///   1              2              3     4                5
/// ```
/// Here is a break-down of the different parts.
///
/// 1. is a counter/label for the data element.
/// 2. is a human readable name for the value in that data element.
/// 3. is a Status, which defines, whether a data element is required or  
/// 4. defines the format, in this case alphanumeric and at most 14 characters long.
/// 5. is a comment describing the meaning and content of the element further
///
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataElement {
    pub label: String,
    pub name: String,
    pub st: St,
    pub format: Format,
    pub length: usize,
    pub usage: Usage,
}

impl DataElement {
    /// Returns whether this element is a qualifier data element.
    ///
    /// Typically, this can be gathered from the name of the data
    /// element.
    ///
    /// ## Example
    ///
    ///
    pub fn is_qualifier(&self) -> bool {
        self.name.contains("Qualifier") || self.name.contains("qualifier")
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Usage {
    Text { comment: Option<String> },
    Integer { comment: Option<String> },
    Decimal { comment: Option<String> },
    OneOf { choices: Vec<Choice>, comment: Option<String> },
    Static { value: Choice, comment: Option<String> },
}

/// The status of a segment (group), composite or data element.
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone, Copy)]
pub enum St {
    /// M (Mandatory) means that a data element is mandatory. A data element
    /// is considered missing, if it does not contain any characters. A composite
    /// is considered missing, if it does not contain any data elements. A
    /// segment is considered missing, if does not exist.
    M,
    /// R (Required) means, that the segment, composite or data element is required.
    /// The same rules, as with M apply.
    R,
    /// O (Optional) means, that the segment, composite or data element is not required.
    O,
    /// D (Dependent) means, that the status of a segment, composite or data
    /// element depends on the use case or another segment, composite or data element.
    D,
    /// C (Dependent) means, that the status of a segment, composite or data
    /// element depends on the use case or another segment, composite or data element.
    C,
    /// N (NotUsed) means, that the segment, composite or data element depends on
    /// should not be used.
    N,
}

impl St {
    /// Returns whether this `St` means, that the segment, composite or
    /// data element in question is optional.
    ///
    /// From the perspective of a MIG, that is the case, when it is either
    /// O, C or D.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(St::O.is_optional(), true)
    /// assert_eq!(St::C.is_optional(), true)
    /// assert_eq!(St::D.is_optional(), true)
    /// ```
    pub fn is_optional(&self) -> bool {
        self == &St::O || self == &St::C || self == &St::D
    }

    /// Returns, whether this `St` means, that the segment, composite, or
    /// data element in question is required.
    ///
    /// From the perspective of a MIG, that is the case, when it is either
    /// M or R.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(St::M.is_required(), true)
    /// assert_eq!(St::R.is_required(), true)
    /// ```
    pub fn is_required(&self) -> bool {
        self == &St::R || self == &St::M
    }

    /// Returns, whether this `St` means, that the segment, composite, or
    /// data element in question shall not be used.
    ///
    /// From the perspective of a MIG, that is the case, when it is N. For
    /// data elements and composites, this means the content may be empty.
    /// For segments, it means it shall not exist.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(St::N.is_not_used(), true)
    /// ```
    pub fn is_not_used(&self) -> bool {
        self == &St::N
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct Choice {
    pub value: String,
    pub semantics: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Clone, Copy)]
pub enum Format {
    Alphanumeric(Size),
    Alpha(Size),
    Numeric(Size),
}

#[derive(Debug, Eq, PartialEq, Serialize, Clone, Copy)]
pub enum Size {
    Exactly,
    AtMost,
}

struct FormatVisitor;

impl<'de> Visitor<'de> for FormatVisitor {
    type Value = Format;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a format value an, a, n, an.., a.., n..")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match value {
            "an" => Ok(Format::Alphanumeric(Size::Exactly)),
            "an.." => Ok(Format::Alphanumeric(Size::AtMost)),
            "a" => Ok(Format::Alpha(Size::Exactly)),
            "a.." => Ok(Format::Alpha(Size::AtMost)),
            "n" => Ok(Format::Numeric(Size::Exactly)),
            "n.." => Ok(Format::Numeric(Size::AtMost)),
            _ => Err(E::custom(format!("format out of range: {}", value))),
        }
    }
}

impl<'de> Deserialize<'de> for Format {
    fn deserialize<D>(deserializer: D) -> Result<Format, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(FormatVisitor)
    }
}
