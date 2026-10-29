//! Independent callers, DRAFT. The generator must create implementation.rs.
//! Its execution predicate may contain only initialization and actual permitted
//! transitions; unproved reachable-state invariants are NOT allowed premises.
#[path = "/home/nitro/Desktop/raft_verus/runs/sol-high-01/requirements/targets.rs"]
mod targets;
#[path = "/home/nitro/Desktop/raft_verus/runs/sol-high-01/madraft/proof/implementation.rs"]
mod implementation;
use vstd::prelude::*;

verus! {
pub proof fn election_safety(run: implementation::Execution)
    requires implementation::execution(run),
    ensures targets::election_safety(implementation::observe(run)),
{ implementation::prove_election_safety(run); }

pub proof fn leader_append_only(run: implementation::Execution)
    requires implementation::execution(run),
    ensures targets::leader_append_only(implementation::observe(run)),
{ implementation::prove_leader_append_only(run); }

pub proof fn log_matching(run: implementation::Execution)
    requires implementation::execution(run),
    ensures targets::log_matching(implementation::observe(run)),
{ implementation::prove_log_matching(run); }

pub proof fn leader_completeness(run: implementation::Execution)
    requires implementation::execution(run),
    ensures targets::leader_completeness(implementation::observe(run)),
{ implementation::prove_leader_completeness(run); }

pub proof fn state_machine_safety(run: implementation::Execution)
    requires implementation::execution(run),
    ensures targets::state_machine_safety(implementation::observe(run)),
{ implementation::prove_state_machine_safety(run); }

pub proof fn snapshot_and_observation_shape(run: implementation::Execution)
    requires implementation::execution(run),
    ensures implementation::refines_executable(run),
        targets::shape(implementation::observe(run)),
        targets::snapshot_fidelity(implementation::observe(run), implementation::applications()),
{ implementation::prove_observation_and_snapshots(run); }
} // verus!
