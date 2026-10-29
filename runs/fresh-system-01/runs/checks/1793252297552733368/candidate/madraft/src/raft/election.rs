//! Synchronous election decisions shared by the host and Verus.
use vstd::prelude::*;

verus! {
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role { Follower, Candidate, Leader }

#[derive(Debug, Clone)]
pub struct Election {
    pub term: u64,
    pub role: Role,
    pub voted_for: Option<usize>,
    pub me: usize,
    pub members: usize,
    pub last_term: u64,
    pub last_index: u64,
    pub grants: Vec<bool>,
    pub grant_count: usize,
}

pub struct Image {
    pub term: u64, pub role: Role, pub vote: Option<usize>,
    pub me: usize, pub members: usize,
    pub last_term: u64, pub last_index: u64,
    pub grants: Seq<bool>, pub count: usize,
}

impl Election {
    pub open spec fn image(&self) -> Image {
        Image { term: self.term, role: self.role, vote: self.voted_for,
            me: self.me, members: self.members, last_term: self.last_term,
            last_index: self.last_index, grants: self.grants@, count: self.grant_count }
    }
    pub open spec fn valid(&self) -> bool {
        self.members > 0 && self.me < self.members && self.grants.len() == self.members
        && self.grant_count <= self.members
        && (self.voted_for.is_some() ==> self.voted_for.unwrap() < self.members)
    }
    pub fn new(members: usize, me: usize) -> (r: Option<Self>)
        ensures members > 0 && me < members ==> r.is_some() && r.unwrap().valid()
            && r.unwrap().term == 0 && r.unwrap().role == Role::Follower
            && r.unwrap().voted_for.is_none(),
            !(members > 0 && me < members) ==> r.is_none(),
    {
        if members == 0 || me >= members { return None; }
        let mut grants = Vec::new();
        let mut i = 0;
        while i < members
            invariant i <= members, grants.len() == i,
            decreases members - i,
        { grants.push(false); i += 1; }
        Some(Self { term: 0, role: Role::Follower, voted_for: None,
            me, members, last_term: 0, last_index: 0, grants, grant_count: 0 })
    }
    pub fn is_leader(&self) -> (r: bool)
        ensures r == (self.role == Role::Leader),
    { matches!(self.role, Role::Leader) }

    pub fn observe_term(&mut self, term: u64)
        requires old(self).valid(),
        ensures self.valid(), self.term == if term > old(self).term { term } else { old(self).term },
            term <= old(self).term ==> self.image() == old(self).image(),
            term > old(self).term ==> self.role == Role::Follower && self.voted_for.is_none(),
            self.me == old(self).me, self.members == old(self).members,
            self.last_term == old(self).last_term, self.last_index == old(self).last_index,
    {
        if term > self.term {
            self.term = term;
            self.role = Role::Follower;
            self.voted_for = None;
        }
    }
    pub fn campaign(&mut self) -> (ok: bool)
        requires old(self).valid(),
        ensures self.valid(), self.me == old(self).me, self.members == old(self).members,
            self.last_term == old(self).last_term, self.last_index == old(self).last_index,
            ok ==> self.term == old(self).term + 1 && self.role == Role::Candidate
                && self.voted_for == Some(self.me) && self.grant_count == 1,
            !ok ==> self.image() == old(self).image(),
            ok == (old(self).term < u64::MAX),
    {
        if self.term == u64::MAX { return false; }
        self.term += 1;
        self.role = Role::Candidate;
        self.voted_for = Some(self.me);
        let mut i = 0;
        while i < self.members
            invariant self.valid(), i <= self.members,
            decreases self.members - i,
        { self.grants.set(i, false); i += 1; }
        self.grants.set(self.me, true);
        self.grant_count = 1;
        true
    }
    pub fn request_vote(&mut self, term: u64, candidate: usize, last_term: u64, last_index: u64) -> (granted: bool)
        requires old(self).valid(),
        ensures self.valid(), self.term >= old(self).term,
            self.me == old(self).me, self.members == old(self).members,
            self.last_term == old(self).last_term, self.last_index == old(self).last_index,
            granted ==> candidate < self.members && term == self.term
                && self.voted_for == Some(candidate)
                && (last_term > self.last_term || (last_term == self.last_term && last_index >= self.last_index)),
            self.term == old(self).term && old(self).voted_for.is_some() ==> self.voted_for == old(self).voted_for,
            candidate >= old(self).members ==> self.image() == old(self).image(),
    {
        if candidate >= self.members { return false; }
        self.observe_term(term);
        if term != self.term { return false; }
        if last_term < self.last_term || (last_term == self.last_term && last_index < self.last_index) { return false; }
        match self.voted_for {
            Some(id) => { if id != candidate { return false; } }
            None => {}
        }
        self.voted_for = Some(candidate);
        true
    }
    pub fn receive_grant(&mut self, term: u64, peer: usize, granted: bool) -> (elected: bool)
        requires old(self).valid(),
        ensures self.valid(), self.term >= old(self).term,
            self.me == old(self).me, self.members == old(self).members,
            self.last_term == old(self).last_term, self.last_index == old(self).last_index,
            elected ==> old(self).role == Role::Candidate && self.role == Role::Leader
                && self.term == term && self.grant_count > self.members / 2,
            peer >= old(self).members ==> self.image() == old(self).image(),
    {
        if peer >= self.members { return false; }
        self.observe_term(term);
        if term != self.term || !matches!(self.role, Role::Candidate) { return false; }
        if granted && !self.grants[peer] && self.grant_count < self.members {
            self.grants.set(peer, true);
            self.grant_count += 1;
        }
        if self.grant_count > self.members / 2 {
            self.role = Role::Leader;
            return true;
        }
        false
    }
    pub fn heartbeat(&mut self, term: u64, leader: usize) -> (accepted: bool)
        requires old(self).valid(),
        ensures self.valid(), self.term >= old(self).term,
            self.me == old(self).me, self.members == old(self).members,
            self.last_term == old(self).last_term, self.last_index == old(self).last_index,
            accepted ==> self.term == term && self.role == Role::Follower,
            self.term == old(self).term ==> self.voted_for == old(self).voted_for,
            leader >= old(self).members ==> self.image() == old(self).image(),
    {
        if leader >= self.members { return false; }
        self.observe_term(term);
        if term != self.term { return false; }
        self.role = Role::Follower;
        true
    }
}
}
