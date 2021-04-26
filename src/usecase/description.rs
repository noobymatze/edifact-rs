use crate::mig::either::Either;

/// A manual defines multiple use cases for a given message
/// (e.g. UTILMD) and version (e.g. 5.1a).
pub struct Manual {
    message: String,
    version: String,
    use_cases: Vec<UseCase>,
}

///
pub struct UseCase {
    pub ident: Option<Identifier>,
    pub data: Vec<Either<Segmentgroup, Segment>>,
}

/// An `Identifier` is used to identify a use case, contained in
/// one or more EDIFACT messages of certain types.
pub struct Identifier {
    pub value: String,
}

pub struct Segment {
    pub order: u16,
    pub name: String,
    pub necessities: Vec<Necessity>,
    pub elements: Vec<DataElement>,
}

pub struct Segmentgroup {
    pub order: u16,
    pub name: String,
    pub necessities: Vec<Necessity>,
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
    Ref(u16),
    Cond(Op, Condition, Condition),
}
