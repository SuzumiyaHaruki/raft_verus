// Existing upstream state and accessor, relocated for same-source tool checking.
use vstd::prelude::*;

verus! {
/// State of a raft peer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    pub term: u64,
    pub role: Role,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Follower,
    Candidate,
    Leader,
}

impl State {
    pub fn is_leader(&self) -> (result: bool)
        ensures result == (self.role is Leader),
    {
        false
    }
}

}

impl Default for Role {
    fn default() -> Self {
        Role::Follower
    }
}

impl Default for State {
    fn default() -> Self { Self { term: 0, role: Role::default() } }
}
