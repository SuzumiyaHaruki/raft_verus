//! Actual per-incarnation constructor/handler-to-history connection.
//! PARTIAL: not yet the full cluster adapters consumed by requirements/final.rs.
//! No global theorem or crash-history completeness is claimed by this entry.
#[path = "durable_vote_reply.rs"]
pub mod durable_vote_reply;
pub use durable_vote_reply::storage_correspondence::{protocol, state, storage};
#[path = "../src/raft/execution.rs"]
pub mod executable;
#[path = "../src/raft/election.rs"]
pub mod election;
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

pub open spec fn view(s: RuntimeState) -> State {
    State { nodes: Map::new(|k: NodeKey| s.nodes.contains_key(k),
        |k: NodeKey| s.nodes[k].states.last()) }
}
pub open spec fn initial(s: State) -> bool {
    forall|k: NodeKey| s.nodes.contains_key(k) ==>
        protocol::initial_view(#[trigger] s.nodes[k], s.nodes[k].state.me, s.nodes[k].state.members)
}
pub open spec fn runtime_initial(s: RuntimeState) -> bool {
    s.prior_incarnations.dom().is_empty()
    && forall|k: NodeKey| s.nodes.contains_key(k) ==> {
        let h = #[trigger] s.nodes[k];
        executable::valid(h) && h.inputs.len() == 0
        && h.origin == executable::Origin::Fresh(h.states[0].state.me, h.states[0].state.members)
    }
}
pub proof fn initial_registry_projection(s: RuntimeState)
    requires runtime_initial(s),
    ensures initial(view(s)),
{
    assert forall|k: NodeKey| view(s).nodes.contains_key(k) implies
        protocol::initial_view(#[trigger] view(s).nodes[k],
            view(s).nodes[k].state.me, view(s).nodes[k].state.members) by {
        let h = s.nodes[k];
        assert(h.states.len() == 1);
        assert(h.states.last() == h.states[0]);
    }
}

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
