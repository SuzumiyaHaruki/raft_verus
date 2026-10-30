//! Conditional vote recovery: storage provenance remains an outer obligation.
#[path = "../src/raft/state.rs"]
pub mod state;
use vstd::prelude::*;
verus! {
// This call sequence uses the real decoder-side validation and request handler.
// The premise names the recorded earlier vote, not an arbitrary success flag.
pub fn restore_then_vote(me: usize, members: usize, term: u64, voter: usize,
    request: state::VoteRequest) -> (result: Option<(state::State, state::VoteReply)>)
    ensures result.is_some() ==> state::well_formed(result.unwrap().0),
        result.is_some() && request.term == term && request.candidate != voter
            ==> !result.unwrap().1.granted,
{
    match state::State::restore_vote(me, members, term, Some(voter)) {
        None => None,
        Some(mut s) => {
            let reply = s.request_vote(request);
            Some((s, reply))
        }
    }
}
}
