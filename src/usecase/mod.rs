use crate::mig::either::Either;

mod description;

pub struct Identifier {
    pub path: Vec<String>,
    pub value: String,
}

pub struct UseCase {
    pub ident: Identifier,
}

pub enum Necessity {
    Must(Option<Condition>),
    Should(Option<Condition>),
    Can(Option<Condition>),
}

pub enum Op {
    X,
    U,
    O,
}

pub enum Condition {
    Ref(u16, String),
    Cond(Op, Condition, Condition),
}

pub struct DataElement {
    pub op: Option<Op>,
    pub cond: Option<Condition>,
    pub values: Vec<Value>,
}

pub struct Value {
    pub code: String,
    pub op: Option<(Op, Option<Condition>)>,
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
