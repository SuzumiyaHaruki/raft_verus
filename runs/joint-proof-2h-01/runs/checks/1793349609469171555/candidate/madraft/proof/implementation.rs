//! Actual per-incarnation constructor/handler-to-history connection.
//! PARTIAL: not yet the full cluster adapters consumed by requirements/final.rs.
//! No global theorem or crash-history completeness is claimed by this entry.
#[path = "durable_vote_reply.rs"]
pub mod durable_vote_reply;
pub use durable_vote_reply::storage_correspondence::{protocol, state, storage, election};
#[path = "../src/raft/execution.rs"]
pub mod executable;
#[path = "../src/raft/wire.rs"]
pub mod wire;
#[path = "../../requirements/support.rs"]
pub mod segment_support;
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
pub enum Message { VoteRequest(state::VoteRequest), VoteReply(state::VoteRequest, state::VoteReply) }
pub struct State {
    pub nodes: Map<NodeKey, protocol::OperationView>,
}
pub struct RuntimeState {
    pub nodes: Map<NodeKey, executable::History>,
    pub prior_incarnations: Map<NodeKey, Seq<executable::History>>,
}
pub enum Input {
    Local(NodeKey, executable::Input),
    RequestSubmission(NodeKey, wire::RequestView),
    ResponseSubmission(NodeKey, wire::ResponseView),
    Delivery(NodeKey, segment_support::Delivery<Message>),
}
pub enum Output {
    Local(NodeKey, executable::Output),
    RequestSubmission(NodeKey, wire::RequestView),
    ResponseSubmission(NodeKey, wire::ResponseView),
    Delivered(NodeKey),
}

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

verus! {
// These definitions cover local journal and RPC boundary segments only.
// Submission is an exact host action descriptor, NOT proof that a packet was
// emitted. Runtime/host dispatch and crash steps remain absent and OPEN.
pub open spec fn request_envelope(k: NodeKey, q: wire::RequestView) -> segment_support::Envelope<Message> {
    segment_support::Envelope { group: k.group, sender: q.sender as nat,
        recipient: q.recipient as nat, body: Message::VoteRequest(q.request) }
}
pub open spec fn response_envelope(k: NodeKey, q: wire::ResponseView) -> segment_support::Envelope<Message> {
    segment_support::Envelope { group: k.group, sender: q.sender as nat,
        recipient: q.recipient as nat, body: Message::VoteReply(q.request, q.reply) }
}
pub open spec fn receive(i: Input) -> Option<segment_support::Delivery<Message>> {
    match i { Input::Delivery(_, d) => Some(d), _ => None }
}
pub open spec fn emit(o: Output) -> Seq<segment_support::Envelope<Message>> {
    match o {
        Output::RequestSubmission(k, q) => seq![request_envelope(k,q)],
        Output::ResponseSubmission(k, q) => seq![response_envelope(k,q)],
        _ => Seq::empty(),
    }
}
pub open spec fn step(s: State, i: Input, t: State, o: Output) -> bool {
    match (i,o) {
        (Input::Local(k, call), Output::Local(j, result)) => k == j && s.nodes.contains_key(k)
            && t.nodes.contains_key(k) && executable::step(s.nodes[k], call, t.nodes[k], result)
            && t.nodes == s.nodes.insert(k, t.nodes[k]),
        (Input::RequestSubmission(k, q), Output::RequestSubmission(j, out)) =>
            j == k && out == q && k.node == q.sender && t == s,
        (Input::ResponseSubmission(k, q), Output::ResponseSubmission(j, out)) =>
            j == k && out == q && k.node == q.sender && t == s,
        (Input::Delivery(k, d), Output::Delivered(j)) => j == k
            && d.message.group == k.group && d.message.recipient == k.node && t == s,
        _ => false,
    }
}
// Actual shared-source extraction: the caller uses RequestSend::args and the
// exact same descriptor here. This proves value correspondence, not transport.
pub proof fn request_descriptor_connection(call: election::VoteCall, sent: wire::RequestSend,
    k: NodeKey, s: State)
    requires sent.view().request == call.request(), sent.view().recipient == call.peer(),
        sent.view().sender == call.request().candidate, k.node == sent.view().sender,
    ensures step(s, Input::RequestSubmission(k, sent.view()), s, Output::RequestSubmission(k, sent.view())),
        emit(Output::RequestSubmission(k, sent.view()))[0].body == Message::VoteRequest(call.request()),
        emit(Output::RequestSubmission(k, sent.view()))[0].recipient == call.peer(),
{}
}
