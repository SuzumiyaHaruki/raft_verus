//! Partial same-source contracts. No global Raft safety theorem is claimed.
#[path = "../src/raft/state.rs"]
mod state;
#[path = "../src/kvraft/sequence.rs"]
mod sequence;
#[path = "../src/shardkv/transfer.rs"]
mod transfer;
use state::{Ballots, State};
use vstd::prelude::*;
verus! {
// Exercise the actual core call composition, including stale and duplicate
// replies. Authentic reply provenance and async storage are not proved here.
pub fn election_core_calls() {
    let mut s = State::fresh();
    let began = s.campaign(0);
    assert(began && s.term == 1 && s.voted_for == Some(0u64));
    let mut ballots = Ballots::new(1, 3, 0);
    ballots.record(1, 0, true);
    ballots.record(1, 1, true);
    ballots.record(1, 1, true);
    let _ = s.elected(&ballots);
    let no = s.vote(1, 2, true);
    assert(!no && s.voted_for == Some(0u64));
    let _ = s.accept_leader(2);
    assert(s.term == 2 && s.voted_for == None::<u64>);
    let yes = s.vote(2, 2, true);
    assert(yes && s.voted_for == Some(2u64));
    let no2 = s.vote(2, 1, true);
    assert(!no2 && s.voted_for == Some(2u64));
}

pub fn recovery_core_calls() {
    let mut before = State::fresh();
    let yes = before.vote(7, 1, true);
    assert(yes);
    let fresh = State::fresh();
    let record = fresh.restore_successor(before.term, before.voted_for, 3);
    assert(record.is_some());
    let mut restarted = record.unwrap();
    let no = restarted.vote(7, 2, true);
    assert(!no && restarted.voted_for == Some(1u64));
    let conflict = restarted.restore_successor(7, Some(2), 3);
    assert(conflict.is_none());
}

pub fn term_exhaustion() {
    let mut s = State::recovered(u64::MAX, Some(3));
    let began = s.campaign(1);
    assert(!began && s.term == u64::MAX && s.voted_for == Some(3u64));
}
}
