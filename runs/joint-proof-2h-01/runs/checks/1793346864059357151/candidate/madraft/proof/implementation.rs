//! Actual per-incarnation constructor/handler-to-history connection.
//! PARTIAL: not yet the full cluster adapters consumed by requirements/final.rs.
//! No global theorem or crash-history completeness is claimed by this entry.
#[path = "durable_vote_reply.rs"]
pub mod durable_vote_reply;
pub use durable_vote_reply::storage_correspondence::{protocol, state, storage};
#[path = "../src/raft/execution.rs"]
pub mod executable;
use vstd::prelude::*;
verus! {
pub proof fn actual_runtime_is_connected(r: executable::Runtime)
    requires r.invariant(),
    ensures executable::valid(r.history()),
        r.history().states.last() == r.view(),
        protocol::view_invariant(r.view()),
{ r.connected(); }
}
