#[path = "../../requirements/support.rs"]
pub mod support;
// Parent theorem over histories maintained by the ACTUAL executable Runtime.
// The Runtime constructors establish invariant, and its vote and candidate-entry mutators preserve it.
#[path = "implementation.rs"]
pub mod implementation;
use implementation::{executable as e, protocol as p, state};
use vstd::prelude::*;
verus! {
pub proof fn state_at(h: e::History, k: int)
    requires e::valid(h), 0 <= k < h.states.len(),
    ensures p::view_invariant(h.states[k]),
    decreases k,
{
    if k > 0 {
        state_at(h, k-1);
        assert(e::step(h.states[k-1], h.inputs[k-1], h.states[k], h.outputs[k-1]));
    }
}

pub proof fn pending_origin(h: e::History, k: int)
    requires e::valid(h), 0 <= k < h.states.len(), h.states[k].pending.is_some(),
    ensures exists|j: int| 0 <= j < k
        && (h.inputs[j] == e::Input::Request(h.states[k].pending.unwrap().2)
            || h.inputs[j] == e::Input::SelfVote(h.states[k].pending.unwrap().2))
        && h.states[j].state == h.states[k].pending.unwrap().4,
    decreases k,
{
    assert(k > 0);
    assert(e::step(h.states[k-1], h.inputs[k-1], h.states[k], h.outputs[k-1]));
    if h.states[k-1].pending == h.states[k].pending {
        pending_origin(h, k-1);
    } else {
        assert(h.inputs[k-1] == e::Input::Request(h.states[k].pending.unwrap().2)
            || h.inputs[k-1] == e::Input::SelfVote(h.states[k].pending.unwrap().2));
        assert(h.states[k-1].state == h.states[k].pending.unwrap().4);
        assert(exists|j: int| 0 <= j < k
            && (h.inputs[j] == e::Input::Request(h.states[k].pending.unwrap().2)
            || h.inputs[j] == e::Input::SelfVote(h.states[k].pending.unwrap().2))
            && h.states[j].state == h.states[k].pending.unwrap().4);
    }
}

pub proof fn every_actual_prefix(r: e::Runtime, k: int)
    requires r.invariant(), 0 <= k < r.history().states.len(),
    ensures e::valid(r.history()), p::view_invariant(r.history().states[k]),
{
    implementation::actual_runtime_is_connected(r);
    state_at(r.history(), k);
}

pub proof fn every_actual_released_reply(r: e::Runtime, k: int,
    id: u64, image: p::VoteImage, reply: state::VoteReply)
    requires r.invariant(), 0 <= k < r.history().inputs.len(),
        r.history().inputs[k] == e::Input::Release(id, image),
        r.history().outputs[k] == e::Output::Reply(Some(reply)),
    ensures image == p::image(r.history().states[k+1].state),
        exists|j: int| 0 <= j < k
            && (r.history().inputs[j] == e::Input::Request(r.history().states[k].pending.unwrap().2)
                || r.history().inputs[j] == e::Input::SelfVote(r.history().states[k].pending.unwrap().2)),
        r.history().states[k].pending.is_some(),
        state::vote_step(r.history().states[k].pending.unwrap().4,
            r.history().states[k].pending.unwrap().2, r.history().states[k+1].state, reply),
{
    implementation::actual_runtime_is_connected(r);
    let h = r.history();
    state_at(h, k);
    assert(e::step(h.states[k], h.inputs[k], h.states[k+1], h.outputs[k]));
    implementation::durable_vote_reply::storage_correspondence::vote_operation_history::released_reply(
        h.states[k], id, image, h.states[k+1], reply);
    pending_origin(h, k);
}

// Parent use of the actual candidate-entry record. Storage correspondence is
// still a separate conditional obligation; this theorem makes no disk claim.
pub proof fn actual_candidate_entry(r: e::Runtime, k: int, term: u64)
    requires r.invariant(), 0 <= k < r.history().inputs.len(),
        r.history().inputs[k] == e::Input::Campaign(term),
        r.history().outputs[k] == e::Output::Campaign(true),
    ensures r.history().states[k].pending.is_none(),
        r.history().states[k].state.voted_for == Some(r.history().states[k].state.me),
        r.history().states[k+1].state.term == term,
        r.history().states[k+1].state.role == state::Role::Candidate,
        p::image(r.history().states[k+1].state) == p::image(r.history().states[k].state),
{
    implementation::actual_runtime_is_connected(r);
    let h = r.history();
    state_at(h, k);
    assert(e::step(h.states[k], h.inputs[k], h.states[k+1], h.outputs[k]));
}

pub proof fn actual_collector_grants(t: implementation::election::Tally)
    requires t.invariant(),
    ensures t.view().voters.no_duplicates(),
        forall|i: int| 0 <= i < t.view().voters.len() ==>
            #[trigger] t.view().voters[i] < t.view().members
            && exists|j: int| 0 <= j < t.view().replies.len()
                && t.view().replies[j].0 == t.view().voters[i]
                && implementation::election::granted(t.view(), t.view().voters[i], t.view().replies[j].1),
{ t.evidence(); }

pub proof fn actual_leader_requires_majority(r: e::Runtime, k: int, votes: implementation::election::TallyView)
    requires r.invariant(), 0 <= k < r.history().inputs.len(),
        r.history().inputs[k] == e::Input::Elect(votes), r.history().outputs[k] == e::Output::Elected(true),
    ensures implementation::election::valid(votes), votes.voters.len() > votes.members / 2,
        votes.request.term == r.history().states[k+1].state.term,
        votes.request.candidate == r.history().states[k+1].state.me,
        votes.members == r.history().states[k+1].state.members,
        r.history().states[k+1].state.role == state::Role::Leader,
{
    implementation::actual_runtime_is_connected(r);
    let h = r.history();
    assert(e::step(h.states[k], h.inputs[k], h.states[k+1], h.outputs[k]));
}

pub proof fn actual_higher_term_steps_down(r: e::Runtime, k: int, peer: usize, reply: state::VoteReply, c: p::TermView)
    requires r.invariant(), 0 <= k < r.history().inputs.len(),
        r.history().inputs[k] == e::Input::HigherReply(peer, reply),
        r.history().outputs[k] == e::Output::TermOp(Some(c)),
    ensures r.history().states[k+1].state.term == reply.term,
        reply.term > r.history().states[k].state.term,
        r.history().states[k+1].state.role == state::Role::Follower,
        r.history().states[k+1].state.voted_for.is_none(),
        p::image(r.history().states[k+1].state) == p::term_image(c),
{
    implementation::actual_runtime_is_connected(r);
    let h = r.history();
    state_at(h, k);
    assert(e::step(h.states[k], h.inputs[k], h.states[k+1], h.outputs[k]));
}

pub proof fn actual_descriptor_submission(call: implementation::election::VoteCall,
    sent: implementation::wire::RequestSend, k: implementation::NodeKey, s: implementation::State)
    requires sent.view().request == call.request(), sent.view().recipient == call.peer(),
        sent.view().sender == call.request().candidate, k.node == sent.view().sender,
    ensures implementation::emit(implementation::Output::RequestSubmission(k, sent.view())).len() == 0,
        implementation::request_envelope(k, sent.view()).recipient == call.peer(),
        crate::support::exchange(Seq::empty(), None,
            implementation::emit(implementation::Output::Emitted(implementation::request_envelope(k,sent.view()))),
            seq![implementation::request_envelope(k,sent.view())]),
{
    implementation::request_descriptor_connection(call, sent, k, s);
}

}
