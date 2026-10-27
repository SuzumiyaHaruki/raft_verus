//! DRAFT generation starting point. STATE-INIT must implement and prove this API.
//! The reject-all seed compiles, but deliberately fails independent acceptance.
use crate::state::{InitError, Node};
#[cfg(verus_keep_ghost)]
use crate::initialization::init_result;
use vstd::prelude::*;

verus! {
impl Node {
    pub fn new(id: u64, members: Vec<u64>) -> (result: Result<Self, InitError>)
        ensures init_result(result, id, members@),
    {
        let _ = id;
        let _ = members;
        Err(InitError::InvalidMembership)
    }
}
}
