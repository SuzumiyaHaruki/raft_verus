//! DRAFT observable state vocabulary. This is not the internal Node layout.
use crate::types::Entry;
use crate::io::{OperationId, Message};
use vstd::prelude::*;
pub use crate::node_state::Node;

verus! {
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DurableState {
    pub term: u64,
    pub voted_for: Option<u64>,
    pub log: Vec<Entry>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Role { Follower, Candidate, Leader }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InitError { InvalidMembership }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingWrite {
    pub operation: OperationId,
    pub reply: Option<Message>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Observation {
    pub id: u64,
    pub members: Vec<u64>,
    pub hard: DurableState,
    pub role: Role,
    pub commit: u64,
    pub delivered: u64,
    pub application_cursor: u64,
    pub incarnation: u64,
    pub serial: u64,
    pub deadline: u64,
    pub pending_write: Option<PendingWrite>,
    pub recovery_required: bool,
}
}
