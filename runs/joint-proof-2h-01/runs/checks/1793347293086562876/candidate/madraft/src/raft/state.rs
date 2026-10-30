// Shared executable protocol core, compiled by Cargo and checked by Verus.
use vstd::prelude::*;

verus! {
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    pub term: u64,
    pub role: Role,
    pub voted_for: Option<usize>,
    pub me: usize,
    pub members: usize,
    pub last_index: u64,
    pub last_term: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role { Follower, Candidate, Leader }

#[derive(Debug, Clone, Copy)]
#[cfg_attr(not(verus_keep_ghost), derive(serde::Serialize, serde::Deserialize))]
pub struct VoteRequest {
    pub term: u64,
    pub candidate: usize,
    pub last_index: u64,
    pub last_term: u64,
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(not(verus_keep_ghost), derive(serde::Serialize, serde::Deserialize))]
pub struct VoteReply { pub term: u64, pub granted: bool }

pub open spec fn well_formed(s: State) -> bool {
    s.members > 0 && s.me < s.members
    && (match s.voted_for { Some(n) => n < s.members, None => true })
    && s.last_term <= s.term
}

pub open spec fn initial(s: State, me: usize, members: usize) -> bool {
    well_formed(s) && s.me == me && s.members == members
    && s.term == 0 && s.role == Role::Follower && s.voted_for == None::<usize>
    && s.last_index == 0 && s.last_term == 0
}

pub open spec fn up_to_date(s: State, q: VoteRequest) -> bool {
    q.last_term > s.last_term || (q.last_term == s.last_term && q.last_index >= s.last_index)
}

// Exact local input/state/output relation. It makes no durability assumption.
pub open spec fn vote_step(s: State, q: VoteRequest, t: State, r: VoteReply) -> bool {
    let valid = q.candidate < s.members && q.last_term <= q.term;
    let higher = valid && q.term > s.term;
    let granted = valid && q.term >= s.term && up_to_date(s, q)
        && (higher || s.voted_for == None::<usize> || s.voted_for == Some(q.candidate));
    t.me == s.me && t.members == s.members
    && t.last_index == s.last_index && t.last_term == s.last_term
    && t.term == (if higher { q.term } else { s.term })
    && t.role == (if higher { Role::Follower } else { s.role })
    && t.voted_for == (if granted { Some(q.candidate) } else if higher { None } else { s.voted_for })
    && r.term == t.term && r.granted == granted
}

impl State {
    pub fn new(me: usize, members: usize) -> (result: Option<Self>)
        ensures
            result.is_some() == (members > 0 && me < members),
            result.is_some() ==> initial(result.unwrap(), me, members),
    {
        if members == 0 || me >= members { return None; }
        Some(Self { term: 0, role: Role::Follower, voted_for: None,
            me, members, last_index: 0, last_term: 0 })
    }

    // Recovery validates deployment identity and vote range at the real boundary.
    // Historical durability is a separate obligation of the storage caller.
    pub fn restore_vote(me: usize, members: usize, term: u64, voted_for: Option<usize>)
        -> (result: Option<Self>)
        ensures result.is_some() == (members > 0 && me < members
            && (match voted_for { Some(n) => n < members, None => true })),
            result.is_some() ==> well_formed(result.unwrap())
                && result.unwrap().me == me && result.unwrap().members == members
                && result.unwrap().term == term && result.unwrap().voted_for == voted_for
                && result.unwrap().role == Role::Follower
                && result.unwrap().last_index == 0 && result.unwrap().last_term == 0,
    {
        if members == 0 || me >= members { return None; }
        if let Some(n) = voted_for { if n >= members { return None; } }
        Some(Self { term, role: Role::Follower, voted_for, me, members,
            last_index: 0, last_term: 0 })
    }

    pub fn is_leader(&self) -> (result: bool)
        ensures result == (self.role is Leader),
    { matches!(self.role, Role::Leader) }

    pub fn request_vote(&mut self, q: VoteRequest) -> (reply: VoteReply)
        requires well_formed(*old(self)),
        ensures well_formed(*self), vote_step(*old(self), q, *self, reply),
            self.term >= old(self).term,
            old(self).voted_for.is_some() && self.term == old(self).term
                ==> self.voted_for == old(self).voted_for,
    {
        if q.candidate >= self.members || q.last_term > q.term {
            return VoteReply { term: self.term, granted: false };
        }
        if q.term > self.term {
            self.term = q.term;
            self.role = Role::Follower;
            self.voted_for = None;
        }
        let fresh = q.last_term > self.last_term
            || (q.last_term == self.last_term && q.last_index >= self.last_index);
        let available = match self.voted_for {
            None => true,
            Some(candidate) => candidate == q.candidate,
        };
        let granted = q.term == self.term && fresh && available;
        if granted { self.voted_for = Some(q.candidate); }
        VoteReply { term: self.term, granted }
    }
}

}
