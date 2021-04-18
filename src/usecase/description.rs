use crate::mig::either::Either;

///
pub struct UseCase {
    pub ident: Identifier,
    pub data: Vec<Either<Segmentgroup, Segment>>,
}

/// An `Identifier` is used to identify a use case, contained in
/// one or more EDIFACT messages.
pub struct Identifier {
    pub path: Vec<String>,
    pub value: String,
}

pub struct Segment {
    pub order: u16,
    pub name: String,
    pub properties: Vec<Necessity>,
    pub elements: Vec<DataElement>,
}

pub struct Segmentgroup {
    pub order: u16,
    pub name: String,
    pub properties: Vec<Necessity>,
    pub segments: Vec<Either<Segmentgroup, Segment>>,
}

pub struct DataElement {
    pub label: String,
    pub op: Option<Op>,
    pub cond: Option<Condition>,
    pub values: Vec<Value>,
}

pub struct Value {
    pub code: String,
    pub op: Option<(Op, Option<Condition>)>,
}

/// The `Necessity` describes, whether a `Segment` or `Segmentgroup`
/// *Must*, *Should* or *Can* occur.
pub enum Necessity {
    Must(Option<Condition>),
    Should(Option<Condition>),
    Can(Option<Condition>),
}

/// An `Op` defines logical operations, *xor*, *and* and *or*,
/// which, depending on the context have different meanings.
pub enum Op {
    X,
    U,
    O,
}

pub enum Condition {
    Ref(u16, String),
    Cond(Op, Condition, Condition),
}
