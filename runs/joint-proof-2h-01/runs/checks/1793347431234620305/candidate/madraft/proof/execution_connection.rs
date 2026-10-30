//! Parent theorem over histories maintained by the ACTUAL executable Runtime.
//! The Runtime constructors establish invariant, and its two mutators preserve it.
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
        && h.inputs[j] == e::Input::Request(h.states[k].pending.unwrap().2)
        && h.states[j].state == h.states[k].pending.unwrap().4,
    decreases k,
{
    assert(k > 0);
    assert(e::step(h.states[k-1], h.inputs[k-1], h.states[k], h.outputs[k-1]));
    if h.states[k-1].pending == h.states[k].pending {
        pending_origin(h, k-1);
    } else {
        assert(h.inputs[k-1] == e::Input::Request(h.states[k].pending.unwrap().2));
        assert(h.states[k-1].state == h.states[k].pending.unwrap().4);
        assert(exists|j: int| 0 <= j < k
            && h.inputs[j] == e::Input::Request(h.states[k].pending.unwrap().2)
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
            && r.history().inputs[j] == e::Input::Request(r.history().states[k].pending.unwrap().2),
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
}
