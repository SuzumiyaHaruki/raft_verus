// Consensus decisions shared by Cargo and Verus.
use vstd::prelude::*;

verus! {
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct State {
    pub(super) term: u64,
    pub(super) role: Role,
    pub(super) voted_for: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Role { Follower, Candidate, Leader }

impl State {
    pub(super) fn fresh() -> (s: Self)
        ensures s.term == 0, s.role is Follower, s.voted_for == None::<u64>,
    { Self { term: 0, role: Role::Follower, voted_for: None } }

    pub(super) fn is_leader(&self) -> (result: bool)
        ensures result == (self.role is Leader),
    { matches!(self.role, Role::Leader) }

    pub(super) fn observe_term(&mut self, term: u64)
        ensures self.term == (if term > old(self).term { term } else { old(self).term }),
            term > old(self).term ==> self.role is Follower && self.voted_for == None::<u64>,
            term <= old(self).term ==> *self == *old(self),
    {
        if term > self.term {
            self.term = term;
            self.role = Role::Follower;
            self.voted_for = None;
        }
    }

    pub(super) fn campaign(&mut self, me: u64) -> (started: bool)
        ensures
            old(self).term < u64::MAX ==> started && self.term == old(self).term + 1
                && self.role is Candidate && self.voted_for == Some(me),
            old(self).term == u64::MAX ==> !started && *self == *old(self),
    {
        if self.term == u64::MAX { return false; }
        self.term = self.term + 1;
        self.role = Role::Candidate;
        self.voted_for = Some(me);
        true
    }

    // Log freshness is supplied by the executable caller; its refinement to
    // the eventual replicated log remains a separate obligation.
    pub(super) fn vote(&mut self, term: u64, candidate: u64, fresh: bool) -> (granted: bool)
        ensures self.term >= old(self).term,
            self.term >= term,
            granted ==> self.term == term && self.voted_for == Some(candidate) && fresh,
            self.term == old(self).term && old(self).voted_for.is_some()
                ==> self.voted_for == old(self).voted_for,
            granted && term == old(self).term && old(self).voted_for.is_some()
                ==> old(self).voted_for == Some(candidate),
    {
        self.observe_term(term);
        let permitted = match self.voted_for {
            None => true,
            Some(previous) => previous == candidate,
        };
        if term == self.term && fresh && permitted {
            self.voted_for = Some(candidate);
            true
        } else { false }
    }

    pub(super) fn accept_leader(&mut self, term: u64) -> (accepted: bool)
        ensures self.term >= old(self).term, self.term >= term,
            accepted == (term >= old(self).term),
            accepted ==> self.role is Follower,
            self.term == old(self).term ==> self.voted_for == old(self).voted_for,
    {
        self.observe_term(term);
        if term == self.term { self.role = Role::Follower; true } else { false }
    }

    pub(super) fn elected(&mut self, election_term: u64) -> (changed: bool)
        ensures self.term == old(self).term, self.voted_for == old(self).voted_for,
            changed ==> self.role is Leader && old(self).role is Candidate
                && election_term == self.term,
    {
        if self.term == election_term && matches!(self.role, Role::Candidate) {
            self.role = Role::Leader;
            true
        } else { false }
    }

    // Durable decoding is checked by the host; this method does not assert a
    // relation to historical votes. That recovery relation remains OPEN.
    pub(super) fn recovered(term: u64, voted_for: Option<u64>) -> (s: Self)
        ensures s.term == term, s.voted_for == voted_for, s.role is Follower,
    { Self { term, role: Role::Follower, voted_for } }
}
}

impl Default for State { fn default() -> Self { Self::fresh() } }
