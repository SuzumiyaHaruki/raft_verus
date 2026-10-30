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
use crate::support as segment_support;
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
    pub submitted: Seq<segment_support::Envelope<Message>>,
    pub emitted: Set<nat>,
    pub received: Seq<segment_support::Envelope<Message>>,
}
pub struct RuntimeState {
    pub nodes: Map<NodeKey, executable::History>,
    pub prior_incarnations: Map<NodeKey, Seq<executable::History>>,
    pub submitted: Seq<segment_support::Envelope<Message>>,
    pub emitted: Set<nat>,
    pub received: Seq<segment_support::Envelope<Message>>,
}
pub enum Input {
    Local(NodeKey, executable::Input),
    RequestSubmission(NodeKey, wire::RequestView),
    ResponseSubmission(NodeKey, wire::ResponseView),
    Delivery(NodeKey, segment_support::Delivery<Message>),
    HostEmission(NodeKey, nat),
}
pub enum Output {
    Local(NodeKey, executable::Output),
    RequestSubmission(NodeKey, wire::RequestView),
    ResponseSubmission(NodeKey, wire::ResponseView),
    Delivered(NodeKey),
    Emitted(segment_support::Envelope<Message>),
}

pub open spec fn view(s: RuntimeState) -> State {
    State { nodes: Map::new(|k: NodeKey| s.nodes.contains_key(k),
        |k: NodeKey| s.nodes[k].states.last()), submitted: s.submitted, emitted: s.emitted, received: s.received }
}
pub open spec fn initial(s: State) -> bool {
    s.submitted.len() == 0 && s.emitted.is_empty() && s.received.len() == 0
    && forall|k: NodeKey| s.nodes.contains_key(k) ==>
        protocol::initial_view(#[trigger] s.nodes[k], s.nodes[k].state.me, s.nodes[k].state.members)
}
pub open spec fn runtime_initial(s: RuntimeState) -> bool {
    s.prior_incarnations.dom().is_empty()
    && s.submitted.len() == 0 && s.emitted.is_empty() && s.received.len() == 0
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
    match o { Output::Emitted(message) => seq![message], _ => Seq::empty() }
}
pub open spec fn delivered_request(s: State, k: NodeKey, v: wire::InboundView) -> bool {
    v.recipient == k.node && exists|j: int| 0 <= j < s.received.len()
        && s.received[j].group == k.group && s.received[j].sender == v.sender
        && s.received[j].recipient == v.recipient && s.received[j].body == Message::VoteRequest(v.request)
}
pub open spec fn input_authorized(s: State, k: NodeKey, i: executable::Input) -> bool {
    match i { executable::Input::Received(v) => delivered_request(s,k,v), _ => true }
}
pub open spec fn step(s: State, i: Input, t: State, o: Output) -> bool {
    match (i,o) {
        (Input::Local(k, call), Output::Local(j, result)) => k == j && s.nodes.contains_key(k)
            && t.nodes.contains_key(k) && input_authorized(s,k,call) && executable::step(s.nodes[k], call, t.nodes[k], result)
            && t == State { nodes: s.nodes.insert(k, t.nodes[k]), ..s },
        (Input::RequestSubmission(k, q), Output::RequestSubmission(j, out)) =>
            j == k && out == q && k.node == q.sender
            && t == State { submitted: s.submitted.push(request_envelope(k,q)), ..s },
        (Input::ResponseSubmission(k, q), Output::ResponseSubmission(j, out)) =>
            j == k && out == q && k.node == q.sender
            && t == State { submitted: s.submitted.push(response_envelope(k,q)), ..s },
        (Input::HostEmission(k, slot), Output::Emitted(message)) =>
            slot < s.submitted.len() && !s.emitted.contains(slot)
            && message == s.submitted[slot as int] && message.sender == k.node && message.group == k.group
            && t == State { emitted: s.emitted.insert(slot), ..s },
        (Input::Delivery(k, d), Output::Delivered(j)) => j == k
            && d.message.group == k.group && d.message.recipient == k.node
            && t == State { received: s.received.push(d.message), ..s },
        _ => false,
    }
}
// Shared-source extraction is value correspondence, NOT a transport completion.
pub proof fn request_descriptor_connection(call: election::VoteCall, sent: wire::RequestSend,
    k: NodeKey, s: State)
    requires sent.view().request == call.request(), sent.view().recipient == call.peer(),
        sent.view().sender == call.request().candidate, k.node == sent.view().sender,
    ensures step(s, Input::RequestSubmission(k, sent.view()),
        State { submitted: s.submitted.push(request_envelope(k,sent.view())), ..s },
        Output::RequestSubmission(k, sent.view())),
        emit(Output::RequestSubmission(k, sent.view())).len() == 0,
        request_envelope(k, sent.view()).body == Message::VoteRequest(call.request()),
        request_envelope(k, sent.view()).recipient == call.peer(),
{}
pub proof fn emission_requires_submission(s: State, t: State, k: NodeKey, slot: nat,
    message: segment_support::Envelope<Message>)
    requires step(s, Input::HostEmission(k, slot), t, Output::Emitted(message)),
    ensures slot < s.submitted.len(), s.submitted[slot as int] == message,
        !s.emitted.contains(slot), t.emitted.contains(slot),
        emit(Output::Emitted(message)) == seq![message],
{}
// This partial executable registry relation retains real per-node journal
// extensions. Crash/restart and host-effect adequacy are still unfinished.
pub open spec fn runtime_step(s: RuntimeState, i: Input, t: RuntimeState, o: Output) -> bool {
    match (i,o) {
        (Input::Local(k, call), Output::Local(j, result)) => k == j && s.nodes.contains_key(k) && t.nodes.contains_key(k)
            && input_authorized(view(s),k,call)
            && t.nodes[k].origin == s.nodes[k].origin
            && t.nodes[k].inputs == s.nodes[k].inputs.push(call)
            && t.nodes[k].outputs == s.nodes[k].outputs.push(result)
            && t.nodes[k].states == s.nodes[k].states.push(t.nodes[k].states.last())
            && executable::step(s.nodes[k].states.last(), call, t.nodes[k].states.last(), result)
            && t == RuntimeState { nodes: s.nodes.insert(k,t.nodes[k]), ..s },
        (Input::RequestSubmission(k,q), Output::RequestSubmission(j,out)) => k == j && q == out && k.node == q.sender
            && t == RuntimeState { submitted: s.submitted.push(request_envelope(k,q)), ..s },
        (Input::ResponseSubmission(k,q), Output::ResponseSubmission(j,out)) => k == j && q == out && k.node == q.sender
            && t == RuntimeState { submitted: s.submitted.push(response_envelope(k,q)), ..s },
        (Input::HostEmission(k,slot), Output::Emitted(message)) => slot < s.submitted.len() && !s.emitted.contains(slot)
            && message == s.submitted[slot as int] && message.sender == k.node && message.group == k.group
            && t == RuntimeState { emitted: s.emitted.insert(slot), ..s },
        (Input::Delivery(k,d), Output::Delivered(j)) => j == k && d.message.group == k.group && d.message.recipient == k.node
            && t == RuntimeState { received: s.received.push(d.message), ..s },
        _ => false,
    }
}
pub proof fn received_journal_enters_registry(s: RuntimeState, k: NodeKey, h: executable::History,
    v: wire::InboundView, output: executable::Output)
    requires s.nodes.contains_key(k), executable::valid(h), delivered_request(view(s),k,v),
        h.origin == s.nodes[k].origin,
        h.inputs == s.nodes[k].inputs.push(executable::Input::Received(v)),
        h.outputs == s.nodes[k].outputs.push(output),
        h.states == s.nodes[k].states.push(h.states.last()),
    ensures runtime_step(s, Input::Local(k, executable::Input::Received(v)),
        RuntimeState { nodes: s.nodes.insert(k,h), ..s }, Output::Local(k,output)),
{
    let i = h.inputs.len() as int - 1;
    assert(executable::step(h.states[i], h.inputs[i], h.states[i+1], h.outputs[i]));
    assert(h.states[i] == s.nodes[k].states.last());
}

}
