//! Shared executable election decisions. Host code supplies log boundaries.
use vstd::prelude::*;
verus! {
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role { Follower, Candidate, Leader }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Election { pub term: u64, pub role: Role, pub voted_for: Option<u64> }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Vote { pub term: u64, pub candidate: u64, pub last_term: u64, pub last_index: u64 }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reply { pub term: u64, pub granted: bool }

pub open spec fn fresh() -> Election {
    Election { term: 0, role: Role::Follower, voted_for: None }
}
pub fn new() -> (s: Election) ensures s == fresh() {
    Election { term: 0, role: Role::Follower, voted_for: None }
}
pub open spec fn advanced(s: Election, term: u64) -> Election {
    if term > s.term { Election { term, role: Role::Follower, voted_for: None } } else { s }
}
pub fn advance(s: Election, term: u64) -> (t: Election)
    ensures t == advanced(s, term), t.term >= s.term,
{
    if term > s.term { Election { term, role: Role::Follower, voted_for: None } } else { s }
}
pub open spec fn timeout_result(s: Election, me: u64) -> Election {
    if s.term < u64::MAX { Election { term: (s.term + 1) as u64, role: Role::Candidate, voted_for: Some(me) } }
    else { s }
}
pub fn timeout(s: Election, me: u64) -> (t: Election)
    ensures t == timeout_result(s, me), t.term >= s.term,
{
    if s.term < u64::MAX { Election { term: s.term + 1, role: Role::Candidate, voted_for: Some(me) } } else { s }
}
pub open spec fn grant(s: Election, v: Vote, last_term: u64, last_index: u64) -> bool {
    let a = advanced(s, v.term);
    v.term == a.term && (a.voted_for == None::<u64> || a.voted_for == Some(v.candidate))
        && (v.last_term > last_term || (v.last_term == last_term && v.last_index >= last_index))
}
pub open spec fn vote_result(s: Election, v: Vote, last_term: u64, last_index: u64) -> Election {
    let a = advanced(s, v.term);
    if grant(s, v, last_term, last_index) { Election { voted_for: Some(v.candidate), ..a } } else { a }
}
pub fn request_vote(s: Election, v: Vote, last_term: u64, last_index: u64) -> (r: (Election, Reply))
    ensures r.0 == vote_result(s,v,last_term,last_index),
        r.1 == (Reply { term: r.0.term, granted: grant(s,v,last_term,last_index) }),
        r.0.term >= s.term,
        s.term == r.0.term && s.voted_for.is_some() ==> r.0.voted_for == s.voted_for,
{
    let mut a = advance(s, v.term);
    let ok = v.term == a.term && (a.voted_for == None::<u64> || a.voted_for == Some(v.candidate))
        && (v.last_term > last_term || (v.last_term == last_term && v.last_index >= last_index));
    if ok { a.voted_for = Some(v.candidate); }
    (a, Reply { term: a.term, granted: ok })
}
pub fn make_vote(s: Election, me: u64, last_term: u64, last_index: u64) -> (v: Vote)
    ensures v == (Vote { term: s.term, candidate: me, last_term, last_index }),
{ Vote { term: s.term, candidate: me, last_term, last_index } }
pub proof fn one_vote(s: Election, a: Vote, b: Vote, lt: u64, li: u64)
    requires a.term == s.term, b.term == s.term,
        grant(s,a,lt,li), grant(vote_result(s,a,lt,li),b,lt,li),
    ensures a.candidate == b.candidate,
{}
}
