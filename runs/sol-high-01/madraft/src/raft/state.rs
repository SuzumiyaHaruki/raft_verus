//! Synchronous term/vote decisions used directly by the RPC implementation.
use vstd::prelude::*;

verus! {
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    pub term: u64,
    pub role: Role,
    pub voted_for: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role { Follower, Candidate, Leader }

impl State {
    pub fn initial() -> (s: Self)
        ensures s.term == 0, s.role is Follower, s.voted_for.is_none(),
    { Self { term: 0, role: Role::Follower, voted_for: None } }

    pub fn is_leader(&self) -> (result: bool)
        ensures result == (self.role is Leader),
    { matches!(self.role, Role::Leader) }

    pub fn observe_term(&mut self, term: u64)
        ensures
            self.term == if term > old(self).term { term } else { old(self).term },
            term > old(self).term ==> self.role is Follower && self.voted_for.is_none(),
            term <= old(self).term ==> *self == *old(self),
    {
        if term > self.term {
            self.term = term;
            self.role = Role::Follower;
            self.voted_for = None;
        }
    }

    pub fn campaign(&mut self, me: usize) -> (ok: bool)
        ensures
            ok ==> old(self).term < u64::MAX && self.term == old(self).term + 1
                && self.role is Candidate && self.voted_for == Some(me),
            !ok ==> *self == *old(self) && self.term == u64::MAX,
    {
        if self.term == u64::MAX { return false; }
        self.term = self.term + 1;
        self.role = Role::Candidate;
        self.voted_for = Some(me);
        true
    }

    pub fn accept_leader(&mut self, term: u64, leader: usize, members: usize) -> (ok: bool)
        ensures
            ok == (leader < members && term >= old(self).term),
            self.term >= old(self).term,
            ok ==> self.term == term && self.role is Follower,
            !ok ==> *self == *old(self),
            self.term == old(self).term ==> self.voted_for == old(self).voted_for,
            self.term > old(self).term ==> self.voted_for.is_none(),
    {
        if leader >= members || term < self.term { return false; }
        self.observe_term(term);
        self.role = Role::Follower;
        true
    }

    pub fn win(&mut self, me: usize, count: usize, members: usize) -> (ok: bool)
        ensures
            self.term == old(self).term, self.voted_for == old(self).voted_for,
            ok ==> old(self).role is Candidate && self.role is Leader
                && self.voted_for == Some(me) && me < members
                && count > members / 2 && count <= members && members > 0,
            !ok ==> *self == *old(self),
    {
        if matches!(self.role, Role::Candidate) && self.voted_for == Some(me)
            && me < members && majority(count, members) {
            self.role = Role::Leader;
            true
        } else { false }
    }

    // A grant in an unchanged term cannot replace a different vote. Persistence
    // and quorum intersection are separate obligations of the assembled system.
    pub fn vote(&mut self, term: u64, candidate: usize, members: usize,
        last_term: u64, last_index: u64, local_term: u64, local_index: u64) -> (grant: bool)
        ensures
            self.term >= old(self).term,
            grant ==> self.term == term && candidate < members
                && self.voted_for == Some(candidate)
                && (last_term > local_term || (last_term == local_term && last_index >= local_index)),
            self.term == old(self).term && old(self).voted_for.is_some()
                ==> self.voted_for == old(self).voted_for,
            self.voted_for == old(self).voted_for || self.voted_for.is_none()
                || (self.voted_for == Some(candidate) && candidate < members),
            self.role == if self.term > old(self).term { Role::Follower } else { old(self).role },
    {
        if candidate >= members { return false; }
        self.observe_term(term);
        if term != self.term { return false; }
        let fresh = log_fresh(last_term, last_index, local_term, local_index);
        let available = match self.voted_for { None => true, Some(id) => id == candidate };
        if fresh && available { self.voted_for = Some(candidate); true } else { false }
    }
}

pub open spec fn log_fresh_spec(term: u64, index: u64, local_term: u64, local_index: u64) -> bool {
    term > local_term || (term == local_term && index >= local_index)
}

pub fn log_fresh(term: u64, index: u64, local_term: u64, local_index: u64) -> (r: bool)
    ensures r == (term > local_term || (term == local_term && index >= local_index)),
{ term > local_term || (term == local_term && index >= local_index) }

pub fn majority(count: usize, members: usize) -> (r: bool)
    ensures r == (count > members / 2 && count <= members && members > 0),
{ count > members / 2 && count <= members && members > 0 }

pub fn commit_authorized(role: Role, term: u64, entry_term: u64,
    index: u64, commit: u64, end: u64, count: usize, members: usize) -> (r: bool)
    ensures r == (role is Leader && entry_term == term && commit < index <= end
        && count > members / 2 && count <= members && members > 0),
{
    matches!(role, Role::Leader) && entry_term == term && commit < index && index <= end
        && majority(count, members)
}

pub fn snapshot_authorized(base: u64, index: u64, applied: u64, commit: u64, end: u64) -> (r: bool)
    ensures r == (base < index <= applied && index <= commit && index <= end),
{ base < index && index <= applied && index <= commit && index <= end }

pub fn same_epoch(role: Role, term: u64, sent_term: u64) -> (r: bool)
    ensures r == (role is Leader && term == sent_term),
{ matches!(role, Role::Leader) && term == sent_term }

pub proof fn two_votes_one_term(before: State, after: State, a: usize, b: usize)
    requires before.term == after.term, before.voted_for == Some(a),
        after.voted_for == before.voted_for, after.voted_for == Some(b),
    ensures a == b,
{}
}

impl Default for Role { fn default() -> Self { Role::Follower } }
impl Default for State { fn default() -> Self { Self::initial() } }
