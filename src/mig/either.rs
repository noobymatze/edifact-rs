use serde::{Deserialize, Serialize};

#[derive(
    Copy,
    Clone,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
)]
#[serde(untagged)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}
