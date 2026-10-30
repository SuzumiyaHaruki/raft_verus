//! The actual phase machine called by the RPC wrapper, with local correspondence.
#[path = "initialization.rs"]
pub mod initialization;
pub use initialization::state;
#[path = "../src/raft/election.rs"]
pub mod election;
#[path = "../src/raft/protocol.rs"]
pub mod protocol;
use vstd::prelude::*;
verus! {
// Conditional on the real filesystem completing this submitted operation.
// This lemma supplies no fake completion and does not discharge the async caller.
pub proof fn completed_operation(m: protocol::Machine, op: protocol::WriteVote)
    requires m.invariant(), m.accepts(&op),
    ensures op.contents() == protocol::image(m.state()),
{
    m.matching_operation(&op);
}
}
