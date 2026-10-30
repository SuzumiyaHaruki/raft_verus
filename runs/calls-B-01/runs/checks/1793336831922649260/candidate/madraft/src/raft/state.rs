// Consensus decisions shared by Cargo and Verus.
use vstd::prelude::*;

verus! {
pub(super) fn log_fresh(local_term: u64, local_index: u64, candidate_term: u64, candidate_index: u64) -> (yes: bool)
    ensures yes == (candidate_term > local_term || candidate_term == local_term && candidate_index >= local_index),
{ candidate_term > local_term || candidate_term == local_term && candidate_index >= local_index }

// A vector contains the sentinel at index zero, so its old length is the
// original index of the proposed command. Reserve room for the next boundary.
pub(super) fn next_log_index(length: u64) -> (index: Option<u64>)
    ensures index.is_some() == (0 < length < u64::MAX),
        index.is_some() ==> index.unwrap() == length,
{ if length == 0 || length == u64::MAX { None } else { Some(length) } }

pub(super) open spec fn support_count(matches: Seq<u64>, through: nat, index: u64) -> nat
    recommends through <= matches.len(),
    decreases through,
{
    if through == 0 { 0 } else { support_count(matches, (through-1) as nat, index)
        + if matches[through as int-1] >= index { 1nat } else { 0nat } }
}
pub(super) fn replication_support(matches: &Vec<u64>, index: u64) -> (count: u64)
    ensures count == support_count(matches@, matches.len() as nat, index), count <= matches.len(),
{
    let mut i = 0usize;
    let mut count = 0u64;
    while i < matches.len()
        invariant i <= matches.len(), count <= i,
            count == support_count(matches@, i as nat, index),
        decreases matches.len() - i,
    {
        if matches[i] >= index { count += 1; }
        proof { reveal_with_fuel(support_count, 2); }
        i += 1;
    }
    count
}

pub(super) fn authorized_commit(current: u64, candidate: u64, last: u64, term: u64, entry_term: u64, support: u64, members: u64) -> (next: u64)
    ensures next >= current,
        next != current ==> next == candidate && candidate <= last && entry_term == term
            && members > 0 && support <= members && support > members / 2,
{
    if candidate > current && candidate <= last && entry_term == term
        && members > 0 && support <= members && support > members / 2 { candidate } else { current }
}

pub(super) fn commit_from_matches(current: u64, candidate: u64, last: u64, term: u64, entry_term: u64, matches: &Vec<u64>) -> (next: u64)
    ensures next >= current,
        next != current ==> next == candidate && candidate <= last && entry_term == term
            && matches.len() > 0
            && support_count(matches@, matches.len() as nat, candidate) > matches.len() / 2,
{
    let support = replication_support(matches, candidate);
    authorized_commit(current, candidate, last, term, entry_term, support, matches.len() as u64)
}

pub(super) fn next_application(applied: u64, committed: u64, last: u64) -> (next: Option<u64>)
    ensures next.is_some() == (applied < committed && committed <= last),
        next.is_some() ==> next.unwrap() == applied + 1 && next.unwrap() <= committed,
{
    if applied < committed && committed <= last { Some(applied + 1) } else { None }
}

pub(super) fn can_snapshot(base: u64, index: u64, applied: u64, committed: u64, last: u64) -> (yes: bool)
    ensures yes == (base < index && index <= applied && index <= committed && index <= last),
{ base < index && index <= applied && index <= committed && index <= last }

pub(super) fn follower_commit(current: u64, leader: u64, through: u64) -> (next: u64)
    ensures next >= current, next <= if current > through { current } else { through },
        next > current ==> next <= leader && next <= through,
{
    let offered = if leader < through { leader } else { through };
    if offered > current { offered } else { current }
}

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
            granted == (term >= old(self).term && fresh
                && (term > old(self).term || old(self).voted_for.is_none()
                    || old(self).voted_for == Some(candidate))),
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

    // Validate each decoded durable record against the accepted prefix.
    // Storage provenance and decode fidelity remain host-connection obligations.
    pub(super) fn restore_successor(&self, term: u64, vote: Option<u64>, members: u64) -> (r: Option<Self>)
        ensures r.is_some() ==> r.unwrap().term == term && r.unwrap().voted_for == vote
                && r.unwrap().role is Follower && term >= self.term,
            r.is_some() && term == self.term && self.voted_for.is_some()
                ==> vote == self.voted_for,
            r.is_some() && vote.is_some() ==> vote.unwrap() < members,
            r.is_some() == (term >= self.term
                && (term != self.term || self.voted_for.is_none() || vote == self.voted_for)
                && (vote.is_none() || vote.unwrap() < members)),
    {
        if term < self.term { return None; }
        if term == self.term && self.voted_for.is_some() && vote != self.voted_for { return None; }
        if let Some(v) = vote { if v >= members { return None; } }
        Some(Self::recovered(term, vote))
    }

    // Durable decoding is checked by the host; this method does not assert a
    // relation to historical votes. That recovery relation remains OPEN.
    pub(super) fn recovered(term: u64, voted_for: Option<u64>) -> (s: Self)
        ensures s.term == term, s.voted_for == voted_for, s.role is Follower,
    { Self { term, role: Role::Follower, voted_for } }
}
}

impl Default for State { fn default() -> Self { Self::fresh() } }
