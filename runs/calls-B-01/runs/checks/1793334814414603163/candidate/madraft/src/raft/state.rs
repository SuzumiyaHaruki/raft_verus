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

// The tally is bound to one election and contains distinct member IDs.
// RPC provenance for each acknowledged ID remains an async caller obligation.
pub(super) struct Ballots {
    pub(super) term: u64,
    pub(super) members: usize,
    pub(super) voters: Vec<usize>,
}
impl Ballots {
    pub(super) open spec fn valid(&self) -> bool {
        self.members > 0 && self.voters@.no_duplicates()
        && forall|i: int| 0 <= i < self.voters.len() ==> self.voters@[i] < self.members
    }
    pub(super) fn new(term: u64, members: usize, me: usize) -> (b: Self)
        requires me < members,
        ensures b.valid(), b.term == term, b.members == members,
            b.voters@ == seq![me],
    {
        let mut voters = Vec::new();
        voters.push(me);
        Self { term, members, voters }
    }
    pub(super) fn record(&mut self, peer: usize, term: u64, granted: bool)
        requires old(self).valid(),
        ensures self.valid(), self.term == old(self).term, self.members == old(self).members,
            self.voters@.to_set() == (if peer < old(self).members && term == old(self).term && granted {
                old(self).voters@.to_set().insert(peer)
            } else { old(self).voters@.to_set() }),
    {
        if peer >= self.members || term != self.term || !granted { return; }
        let mut i = 0;
        while i < self.voters.len()
            invariant self.valid(), self.term == old(self).term, self.members == old(self).members,
                self.voters@ == old(self).voters@,
                0 <= i <= self.voters.len(),
                forall|j: int| 0 <= j < i ==> self.voters@[j] != peer,
            decreases self.voters.len() - i,
        {
            if self.voters[i] == peer { return; }
            i += 1;
        }
        self.voters.push(peer);
        proof {
            old(self).voters@.lemma_push_to_set_commute(peer);
            assert(self.voters@ == old(self).voters@.push(peer));
            assert(self.voters@.to_set() =~= old(self).voters@.to_set().insert(peer));
        }
    }
    pub(super) fn majority(&self) -> (yes: bool)
        requires self.valid(),
        ensures yes == (self.voters.len() > self.members / 2),
    { self.voters.len() > self.members / 2 }
}

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
        ensures self.term == (if term > old(self).term { term } else { old(self).term }),
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
        ensures self.term == (if term > old(self).term { term } else { old(self).term }),
            term > old(self).term ==> self.voted_for == None::<u64>,
            accepted == (term >= old(self).term),
            accepted ==> self.role is Follower,
            self.term == old(self).term ==> self.voted_for == old(self).voted_for,
    {
        self.observe_term(term);
        if term == self.term { self.role = Role::Follower; true } else { false }
    }

    pub(super) fn elected(&mut self, ballots: &Ballots) -> (changed: bool)
        requires ballots.valid(),
        ensures self.term == old(self).term, self.voted_for == old(self).voted_for,
            changed ==> self.role is Leader && old(self).role is Candidate
                && ballots.term == self.term && ballots.voters.len() > ballots.members / 2,
    {
        if self.term == ballots.term && matches!(self.role, Role::Candidate) && ballots.majority() {
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
