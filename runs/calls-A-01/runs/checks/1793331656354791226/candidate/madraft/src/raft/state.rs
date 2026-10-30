use vstd::prelude::*;

verus! {
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct State {
    pub(super) term: u64,
    pub(super) role: Role,
    pub(super) vote: Option<u64>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Role { Follower, Candidate, Leader }
impl State {
    pub(super) fn is_leader(&self) -> (result: bool)
        ensures result == (self.role is Leader),
    { matches!(self.role, Role::Leader) }

    pub(super) fn adopt_term(&mut self, term: u64)
        ensures self.term == if term > old(self).term { term } else { old(self).term },
            term <= old(self).term ==> *self == *old(self),
            term > old(self).term ==> self.vote == None && self.role is Follower,
    {
        if term > self.term {
            self.term = term;
            self.vote = None;
            self.role = Role::Follower;
        }
    }

    pub(super) fn campaign(&mut self, me: u64) -> (ok: bool)
        ensures ok ==> self.term == old(self).term + 1 && self.vote == Some(me) && self.role is Candidate,
            !ok ==> *self == *old(self) && self.term == u64::MAX,
    {
        if self.term == u64::MAX { return false; }
        self.term = self.term + 1;
        self.vote = Some(me);
        self.role = Role::Candidate;
        true
    }

    pub(super) fn grant_vote(&mut self, term: u64, candidate: u64, fresh: bool) -> (granted: bool)
        ensures self.term >= old(self).term,
            self.term == old(self).term && old(self).vote.is_some() ==> self.vote == old(self).vote,
            granted ==> self.term == term && self.vote == Some(candidate) && fresh,
            self.term == old(self).term || self.term == term,
    {
        self.adopt_term(term);
        if term == self.term && fresh && (self.vote.is_none() || self.vote == Some(candidate)) {
            self.vote = Some(candidate);
            true
        } else { false }
    }
}
}
impl Default for State {
    fn default() -> Self { Self { term: 0, role: Role::Follower, vote: None } }
}
