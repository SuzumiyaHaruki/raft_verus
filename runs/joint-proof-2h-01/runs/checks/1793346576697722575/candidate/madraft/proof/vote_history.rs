//! Necessary election-safety dependency, not a cluster execution relation.
//! This local trace admits EVERY RequestVote input, without filtering messages.
#[path = "vote_correspondence.rs"]
pub mod vote_correspondence;
use vote_correspondence::initialization::state;
use vstd::prelude::*;
verus! {
pub struct VoteHistory {
    pub states: Seq<state::State>,
    pub inputs: Seq<state::VoteRequest>,
    pub outputs: Seq<state::VoteReply>,
}
pub open spec fn calls(h: VoteHistory) -> bool {
    h.states.len() == h.inputs.len() + 1 && h.outputs.len() == h.inputs.len()
    && state::initial(h.states[0], h.states[0].me, h.states[0].members)
    && forall|k: int| 0 <= k < h.inputs.len() ==>
        state::vote_step(h.states[k], h.inputs[k], h.states[k+1], #[trigger] h.outputs[k])
}

pub proof fn vote_stable_between(h: VoteHistory, a: int, b: int)
    requires calls(h), 0 <= a <= b < h.states.len(),
    ensures h.states[a].term <= h.states[b].term,
        h.states[a].term == h.states[b].term && h.states[a].voted_for.is_some()
            ==> h.states[b].voted_for == h.states[a].voted_for,
    decreases b - a,
{
    if a < b {
        vote_stable_between(h, a, b - 1);
        assert(state::vote_step(h.states[b-1], h.inputs[b-1], h.states[b], h.outputs[b-1]));
    }
}

pub proof fn no_two_vote_recipients(h: VoteHistory, i: int, j: int)
    requires calls(h), 0 <= i < h.inputs.len(), 0 <= j < h.inputs.len(),
        h.outputs[i].granted, h.outputs[j].granted,
        h.inputs[i].term == h.inputs[j].term,
    ensures h.inputs[i].candidate == h.inputs[j].candidate,
{
    assert(state::vote_step(h.states[i], h.inputs[i], h.states[i+1], h.outputs[i]));
    assert(state::vote_step(h.states[j], h.inputs[j], h.states[j+1], h.outputs[j]));
    if i <= j { vote_stable_between(h, i+1, j+1); }
    else { vote_stable_between(h, j+1, i+1); }
}
}
