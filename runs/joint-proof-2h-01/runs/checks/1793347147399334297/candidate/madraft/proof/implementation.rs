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

verus! {
// Cluster adapter data shapes. The global initial/step/provenance definitions
// remain unimplemented, so the fixed caller must still reject this partial entry.
// Per-node journals retain distinct identities and incarnation records.
pub struct NodeKey { pub group: nat, pub node: nat }
pub enum Message { VoteRequest(state::VoteRequest), VoteReply(state::VoteReply) }
pub struct State {
    pub nodes: Map<NodeKey, protocol::OperationView>,
}
pub struct RuntimeState {
    pub nodes: Map<NodeKey, executable::History>,
    pub prior_incarnations: Map<NodeKey, Seq<executable::History>>,
}
pub struct Input { pub who: NodeKey, pub call: executable::Input }
pub struct Output { pub who: NodeKey, pub result: executable::Output }

// Necessary registry projection: uses the actual runtime's maintained journal.
// Network envelopes and disk histories must be incorporated before global step.
pub proof fn register_actual_node(r: executable::Runtime, key: NodeKey,
    nodes: Map<NodeKey, executable::History>)
    requires r.invariant(),
    ensures nodes.insert(key, r.history())[key].states.last() == r.view(),
        executable::valid(nodes.insert(key, r.history())[key]),
{
    actual_runtime_is_connected(r);
}
}
