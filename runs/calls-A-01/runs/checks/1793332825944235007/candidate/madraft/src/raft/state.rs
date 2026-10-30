use vstd::prelude::*;

verus! {
#[derive(Debug)]
pub(super) struct State {
    pub(super) term: u64,
    pub(super) role: Role,
    pub(super) vote: Option<u64>,
    pub(super) votes: Vec<bool>,
    pub(super) me: usize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Role { Follower, Candidate, Leader }

pub open spec fn tally(votes: Seq<bool>, n: nat) -> nat
    recommends n <= votes.len(),
    decreases n,
{
    if n == 0 { 0 } else { tally(votes, (n - 1) as nat) + if votes[n as int - 1] { 1nat } else { 0nat } }
}
impl State {
    pub(super) open spec fn valid(&self) -> bool { self.votes.len() > 0 && self.me < self.votes.len() }
    pub(super) fn new(n: usize, me: usize) -> (result: Option<Self>)
        ensures match result {
            Some(s) => s.valid() && s.term == 0 && s.role is Follower && s.vote == None::<u64>
                && s.me == me && s.votes.len() == n && forall|i: int| 0 <= i < n ==> !s.votes@[i],
            None => n == 0 || me >= n,
        },
    {
        if n == 0 || me >= n { return None; }
        Some(Self { term: 0, role: Role::Follower, vote: None, votes: vec![false; n], me })
    }
    pub(super) fn is_leader(&self) -> (result: bool)
        ensures result == (self.role is Leader),
    { matches!(self.role, Role::Leader) }

    pub(super) fn adopt_term(&mut self, term: u64)
        ensures self.term == if term > old(self).term { term } else { old(self).term },
            self.votes == old(self).votes, self.me == old(self).me,
            term <= old(self).term ==> self.role == old(self).role && self.vote == old(self).vote,
            term > old(self).term ==> self.vote == None::<u64> && self.role is Follower,
    {
        if term > self.term { self.term = term; self.vote = None; self.role = Role::Follower; }
    }

    pub(super) fn campaign(&mut self, me: u64) -> (ok: bool)
        requires old(self).valid(), me == old(self).me,
        ensures self.valid(), self.me == old(self).me, self.votes.len() == old(self).votes.len(),
            ok ==> self.term == old(self).term + 1 && self.vote == Some(me) && self.role is Candidate,
            ok ==> forall|i: int| 0 <= i < self.votes.len() ==> self.votes@[i] == (i == self.me),
            !ok ==> self.term == u64::MAX && self.term == old(self).term
                && self.vote == old(self).vote && self.role == old(self).role && self.votes == old(self).votes,
    {
        if self.term == u64::MAX { return false; }
        self.term = self.term + 1; self.vote = Some(me); self.role = Role::Candidate;
        let mut i = 0;
        while i < self.votes.len()
            invariant self.valid(), self.me == old(self).me, self.votes.len() == old(self).votes.len(),
                0 <= i <= self.votes.len(),
                forall|j: int| 0 <= j < i ==> self.votes@[j] == (j == self.me),
            decreases self.votes.len() - i,
        {
            self.votes.set(i, i == self.me);
            i += 1;
        }
        true
    }

    pub(super) fn grant_vote(&mut self, term: u64, candidate: u64, fresh: bool) -> (granted: bool)
        ensures self.term >= old(self).term, self.votes == old(self).votes, self.me == old(self).me,
            self.term == old(self).term && old(self).vote.is_some() ==> self.vote == old(self).vote,
            granted ==> self.term == term && self.vote == Some(candidate) && fresh && candidate < self.votes.len(),
            self.term == old(self).term || self.term == term,
    {
        if candidate >= self.votes.len() as u64 { return false; }
        self.adopt_term(term);
        if term == self.term && fresh && (self.vote.is_none() || self.vote == Some(candidate)) {
            self.vote = Some(candidate); true
        } else { false }
    }

    pub(super) fn vote_count(&self) -> (count: usize)
        ensures count == tally(self.votes@, self.votes.len() as nat), count <= self.votes.len(),
    {
        let mut count = 0usize;
        let mut i = 0usize;
        while i < self.votes.len()
            invariant i <= self.votes.len(), count <= i,
                count == tally(self.votes@, i as nat),
            decreases self.votes.len() - i,
        {
            if self.votes[i] { count += 1; }
            i += 1;
        }
        count
    }

    pub(super) fn accept_vote(&mut self, peer: usize, request_term: u64, reply_term: u64, granted: bool) -> (elected: bool)
        requires old(self).valid(),
        ensures self.valid(), self.me == old(self).me, self.votes.len() == old(self).votes.len(),
            self.term >= old(self).term,
            self.term == old(self).term && old(self).vote.is_some() ==> self.vote == old(self).vote,
            elected ==> self.role is Leader && self.term == request_term
                && tally(self.votes@, self.votes.len() as nat) > self.votes.len() / 2,
            elected ==> old(self).role is Candidate && granted && reply_term <= request_term,
            !elected && reply_term <= old(self).term ==> self.role == old(self).role,
    {
        self.adopt_term(reply_term);
        if peer < self.votes.len() && self.term == request_term && matches!(self.role, Role::Candidate) && granted {
            self.votes.set(peer, true);
            if self.vote_count() > self.votes.len() / 2 {
                self.role = Role::Leader;
                return true;
            }
        }
        false
    }

    pub(super) fn follow(&mut self, term: u64) -> (accepted: bool)
        ensures self.term >= old(self).term, self.votes == old(self).votes, self.me == old(self).me,
            self.term == old(self).term && old(self).vote.is_some() ==> self.vote == old(self).vote,
            accepted ==> self.term == term && self.role is Follower,
            !accepted ==> term < old(self).term && self.term == old(self).term && self.role == old(self).role,
    {
        if term < self.term { return false; }
        self.adopt_term(term); self.role = Role::Follower; true
    }
}
}
