//! Local executable term/vote contracts only; global election safety remains open.
#[path = "../src/raft/state.rs"]
mod state;
use vstd::prelude::*;
verus! {
// A real call composition: a second distinct candidate cannot obtain this
// node's vote at the same term. Durability and quorum intersection are separate.
fn two_vote_requests(s: &mut state::State, term: u64, a: u64, b: u64) -> (r: (bool, bool))
    requires a != b,
    ensures !(r.0 && r.1),
{
    let first = s.grant_vote(term, a, true);
    let second = s.grant_vote(term, b, true);
    (first, second)
}
}
