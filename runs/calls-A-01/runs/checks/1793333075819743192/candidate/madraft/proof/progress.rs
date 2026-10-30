//! Local executable term/vote contracts only; global election safety remains open.
#[path = "../src/raft/state.rs"]
mod state;
#[path = "../src/raft/log.rs"]
mod log_core;
mod quorum;
#[path = "../src/raft/commit.rs"]
mod commit_core;
use vstd::prelude::*;
verus! {
fn actual_single_member_election() -> (s: state::State)
    ensures s.valid(), s.term == 1, s.vote == Some(0u64), s.role is Leader,
{
    let mut s = state::State::new(1, 0).unwrap();
    let started = s.campaign(0);
    let elected = s.accept_vote(0, 1, 1, true);
    assert(started && elected);
    s
}

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
