//! DRAFT shared data vocabulary only; no Raft behavior is supplied.
use vstd::prelude::*;

verus! {
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RequestId { pub client: u64, pub sequence: u64 }

// Clone is a runtime convenience and is not used by the verified operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Command { Client { request: RequestId, bytes: Vec<u8> }, Noop }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entry { pub term: u64, pub command: Command }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReconcileError { InvalidIndex, ResourceExhausted }

}
