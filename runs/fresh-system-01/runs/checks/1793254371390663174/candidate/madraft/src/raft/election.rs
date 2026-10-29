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

pub enum Input {
    Timeout,
    Vote { term: u64, candidate: usize, last_term: u64, last_index: u64 },
    Reply { term: u64, peer: usize, granted: bool },
    Heartbeat { term: u64, leader: usize },
    Observe { term: u64 },
    Crash,
    Restart,
}
pub open spec fn well_formed(i: Image) -> bool {
    i.members > 0 && i.me < i.members && i.grants.len() == i.members
    && i.count <= i.members && (i.vote.is_some() ==> i.vote.unwrap() < i.members)
}
pub open spec fn advance(i: Image, term: u64) -> Image {
    if term > i.term { Image { term, role: Role::Follower, vote: None, ..i } } else { i }
}
pub open spec fn initial(i: Image) -> bool {
    i.term == 0 && i.role == Role::Follower && i.vote.is_none()
    && i.members > 0 && i.me < i.members && i.last_index == 0 && i.last_term == 0
    && i.count == 0 && i.grants == Seq::new(i.members as nat, |j: int| false)
}
pub open spec fn next(i: Image, input: Input) -> Image {
    match input {
        Input::Timeout => if i.term == u64::MAX { i } else {
            Image { term: (i.term + 1) as u64, role: Role::Candidate, vote: Some(i.me),
                grants: Seq::new(i.members as nat, |j: int| j == i.me), count: 1, ..i }
        },
        Input::Observe { term } => advance(i, term),
        Input::Vote { term, candidate, last_term, last_index } => {
            if candidate >= i.members { i } else {
                let b = advance(i, term);
                if term == b.term && (last_term > b.last_term || (last_term == b.last_term && last_index >= b.last_index))
                    && (b.vote.is_none() || b.vote == Some(candidate)) {
                    Image { vote: Some(candidate), ..b }
                } else { b }
            }
        },
        Input::Reply { term, peer, granted } => {
            if peer >= i.members { i } else {
                let b = advance(i, term);
                if term != b.term || b.role != Role::Candidate { b } else {
                    let add = granted && !b.grants[peer as int] && b.count < b.members;
                    let n = if add { (b.count + 1) as usize } else { b.count };
                    Image { grants: if add { b.grants.update(peer as int, true) } else { b.grants },
                        count: n, role: if n > b.members / 2 { Role::Leader } else { b.role }, ..b }
                }
            }
        },
        Input::Heartbeat { term, leader } => if leader >= i.members { i } else {
            let b = advance(i, term);
            if term == b.term { Image { role: Role::Follower, ..b } } else { b }
        },
        // These are provisional crash abstractions, not yet a storage refinement.
        Input::Crash | Input::Restart => Image { role: Role::Follower, ..i },
    }
}

pub proof fn next_preserves(i: Image, input: Input)
    requires well_formed(i),
    ensures well_formed(next(i, input)),
{
    match input {
        Input::Timeout => {},
        Input::Vote { term, candidate, last_term, last_index } => {},
        Input::Reply { term, peer, granted } => {},
        Input::Heartbeat { term, leader } => {},
        Input::Observe { term } => {},
        Input::Crash | Input::Restart => {},
    }
}

impl Election {
    pub open spec fn image(&self) -> Image {
        Image { term: self.term, role: self.role, vote: self.voted_for,
            me: self.me, members: self.members, last_term: self.last_term,
            last_index: self.last_index, grants: self.grants@, count: self.grant_count }
    }
    pub open spec fn valid(&self) -> bool {
        well_formed(self.image())
    }
    pub fn new(members: usize, me: usize) -> (r: Option<Self>)
        ensures members > 0 && me < members ==> r.is_some() && r.unwrap().valid()
            && r.unwrap().term == 0 && r.unwrap().role == Role::Follower
            && r.unwrap().voted_for.is_none() && initial(r.unwrap().image()),
            !(members > 0 && me < members) ==> r.is_none(),
    {
        if members == 0 || me >= members { return None; }
        let mut grants: Vec<bool> = Vec::new();
        let mut i = 0;
        while i < members
            invariant i <= members, grants.len() == i,
                forall |j: int| 0 <= j < i ==> !grants@[j],
            decreases members - i,
        { grants.push(false); i += 1; }
        proof { assert(grants@ =~= Seq::new(members as nat, |j: int| false)); }
        Some(Self { term: 0, role: Role::Follower, voted_for: None,
            me, members, last_term: 0, last_index: 0, grants, grant_count: 0 })
    }
    pub fn recover(&mut self, term: u64, vote: Option<usize>) -> (ok: bool)
        requires old(self).valid(),
        ensures self.valid(),
            ok ==> self.term == term && self.voted_for == vote && self.role == Role::Follower,
            !ok ==> self.image() == old(self).image(),
            ok == (vote.is_none() || vote.unwrap() < old(self).members),
            self.me == old(self).me, self.members == old(self).members,
            self.last_term == old(self).last_term, self.last_index == old(self).last_index,
    {
        match vote {
            Some(id) => { if id >= self.members { return false; } },
            None => {},
        }
        self.term = term;
        self.voted_for = vote;
        self.role = Role::Follower;
        true
    }
    pub fn is_leader(&self) -> (r: bool)
        ensures r == (self.role == Role::Leader),
    { matches!(self.role, Role::Leader) }

    pub fn observe_term(&mut self, term: u64)
        requires old(self).valid(),
        ensures self.valid(), self.image() == advance(old(self).image(), term),
            self.term == if term > old(self).term { term } else { old(self).term },
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
        ensures self.valid(), self.image() == next(old(self).image(), Input::Timeout),
            self.me == old(self).me, self.members == old(self).members,
            self.last_term == old(self).last_term, self.last_index == old(self).last_index,
            ok ==> self.term == old(self).term + 1 && self.role == Role::Candidate
                && self.voted_for == Some(self.me) && self.grant_count == 1,
            !ok ==> self.image() == old(self).image(),
            ok == (old(self).term < u64::MAX),
    {
        if self.term == u64::MAX { return false; }
        self.term = self.term + 1;
        self.role = Role::Candidate;
        self.voted_for = Some(self.me);
        let mut i = 0;
        while i < self.members
            invariant self.valid(), i <= self.members,
                self.me == old(self).me, self.members == old(self).members,
                self.term == old(self).term + 1, self.role == Role::Candidate,
                self.voted_for == Some(self.me),
                self.last_term == old(self).last_term, self.last_index == old(self).last_index,
                self.grant_count == old(self).grant_count,
                forall |j: int| 0 <= j < i ==> !self.grants@[j],
            decreases self.members - i,
        { self.grants.set(i, false); i += 1; }
        proof {
            assert(self.grants@ == Seq::new(self.members as nat, |j: int| false));
        }
        self.grants.set(self.me, true);
        proof {
            assert(self.grants@ =~= Seq::new(self.members as nat, |j: int| j == self.me));
        }
        self.grant_count = 1;
        true
    }
    pub fn request_vote(&mut self, term: u64, candidate: usize, last_term: u64, last_index: u64) -> (granted: bool)
        requires old(self).valid(),
        ensures self.valid(), self.image() == next(old(self).image(), Input::Vote { term, candidate, last_term, last_index }),
            self.term >= old(self).term,
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
        ensures self.valid(), self.image() == next(old(self).image(), Input::Reply { term, peer, granted }),
            self.term >= old(self).term,
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
            self.grant_count = self.grant_count + 1;
        }
        if self.grant_count > self.members / 2 {
            self.role = Role::Leader;
            return true;
        }
        false
    }
    pub fn heartbeat(&mut self, term: u64, leader: usize) -> (accepted: bool)
        requires old(self).valid(),
        ensures self.valid(), self.image() == next(old(self).image(), Input::Heartbeat { term, leader }),
            self.term >= old(self).term,
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
