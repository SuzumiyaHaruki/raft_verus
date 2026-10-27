//! DRAFT initial-state correspondence only, not a view for arbitrary node steps.
//! Initial pending I/O is covered; peer progress and recovery remain incomplete.
use crate::{model, state};
use crate::representation::log_view;
use vstd::prelude::*;

verus! {
pub open spec fn valid_membership(id: u64, members: Seq<u64>) -> bool {
    members.contains(id) && members.no_duplicates()
}

pub open spec fn initial_node(n: state::Observation, id: u64, members: Seq<u64>) -> bool {
    let initial = model::fresh_node();
    n.id == id && n.members@ == members
    && n.hard.term == initial.hard.term
    && n.hard.voted_for.is_none() && initial.hard.voted_for.is_none()
    && log_view(n.hard.log@) == initial.hard.log
    && n.role == state::Role::Follower && initial.role == model::Role::Follower
    && n.commit == initial.commit && n.delivered == initial.delivered
    && n.application_cursor == initial.application_cursor
    && n.incarnation == initial.incarnation && n.serial == initial.serial
    && n.pending_write.is_none() && initial.pending_write.is_none()
    && !n.recovery_required && initial.alive
    && n.deadline == initial.deadline
}

// Requiring success on EVERY valid input prevents a vacuous reject-all proof.
pub open spec fn init_result(result: Result<state::Observation, state::InitError>,
                            id: u64, members: Seq<u64>) -> bool {
    match result {
        Ok(n) => valid_membership(id, members) && initial_node(n, id, members),
        Err(state::InitError::InvalidMembership) => !valid_membership(id, members),
    }
}
}
