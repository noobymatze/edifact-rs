//! A Message Integration Guide (MIG) is an industry specific specialization of
//! an international EDIFACT interchange standard (Electronic Data Interchange for
//! Administration, Commerce and Transport). EDIFACT interchanges are used to
//! electronically exchange business data between two or more parties.
//!
//! The german energy market uses the subset [edi@energy](https://www.edi-energy.de)
//! to exchange customer information between business partners. This subset defines
//! multiple MIGs, many containing multiple use cases. An example for a use case is
//! a customer changing their energy supplier. In that case the new energy supplier
//! will contact the old energy supplier, requesting data, such as the expiration
//! date of the customers contract with the old energy supplier.

pub mod description;
pub mod either;
pub mod error;
mod decode;
pub mod encode;

use std::io::Read;
use crate::mig::decode::value;


pub fn decode<R: Read>(known: Vec<description::Interchange>, input: &mut R) -> Result<value::Interchange, decode::Error> {
    decode::decode(known, input)
}
