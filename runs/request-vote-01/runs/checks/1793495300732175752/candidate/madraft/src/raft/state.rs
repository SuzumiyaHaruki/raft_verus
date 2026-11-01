// Existing upstream state and accessor, relocated for same-source tool checking.
use vstd::prelude::*;

verus! {
/// State of a raft peer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct State {
    pub(super) term: u64,
    pub(super) role: Role,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Role {
    Follower,
    Candidate,
    Leader,
}

impl State {
    pub(super) fn is_leader(&self) -> (result: bool)
        ensures result == (self.role is Leader),
    {
        matches!(self.role, Role::Leader)
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
