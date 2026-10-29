//! Independent callers, DRAFT. The generator must create implementation.rs.
//! Its execution predicate may contain only initialization and actual permitted
//! transitions; unproved reachable-state invariants are NOT allowed premises.
#[path = "/home/nitro/Desktop/raft_verus/runs/fresh-system-01/runs/checks/1793252628678706932/candidate/requirements/targets.rs"]
mod targets;
#[path = "/home/nitro/Desktop/raft_verus/runs/fresh-system-01/runs/checks/1793252628678706932/candidate/madraft/proof/implementation.rs"]
mod implementation;
use vstd::prelude::*;

verus! {
// obligation: election_safety
pub proof fn election_safety(run: implementation::Execution)
    requires implementation::execution(run),
    ensures targets::election_safety(implementation::observe(run)),
{ implementation::prove_election_safety(run); }

// obligation: leader_append_only
pub proof fn leader_append_only(run: implementation::Execution)
    requires implementation::execution(run),
    ensures targets::leader_append_only(implementation::observe(run)),
{ implementation::prove_leader_append_only(run); }

// obligation: log_matching
pub proof fn log_matching(run: implementation::Execution)
    requires implementation::execution(run),
    ensures targets::log_matching(implementation::observe(run)),
{ implementation::prove_log_matching(run); }

// obligation: leader_completeness
pub proof fn leader_completeness(run: implementation::Execution)
    requires implementation::execution(run),
    ensures targets::leader_completeness(implementation::observe(run)),
{ implementation::prove_leader_completeness(run); }

// obligation: state_machine_safety
pub proof fn state_machine_safety(run: implementation::Execution)
    requires implementation::execution(run),
    ensures targets::state_machine_safety(implementation::observe(run)),
{ implementation::prove_state_machine_safety(run); }

// obligation: execution_link
pub proof fn execution_link(run: implementation::Execution)
    requires implementation::execution(run),
    ensures implementation::refines_executable(run),
{ implementation::prove_observation_and_snapshots(run); }

// obligation: snapshot_observation
pub proof fn snapshot_and_observation_shape(run: implementation::Execution)
    requires implementation::execution(run),
    ensures targets::shape(implementation::observe(run)),
        targets::snapshot_fidelity(implementation::observe(run), implementation::applications()),
{ implementation::prove_observation_and_snapshots(run); }

// obligation: nonempty_execution
// A necessary anti-vacuity witness, not a proof of exhaustive runtime coverage.
pub proof fn nonempty_execution()
    ensures exists|run: implementation::Execution| implementation::execution(run)
        && targets::shape(implementation::observe(run))
        && exists|i: int| 0 <= i < implementation::observe(run).events.len()
            && !(implementation::observe(run).events[i] is Silent),
{
    let run = implementation::execution_witness();
    assert(implementation::execution(run));
    assert(targets::shape(implementation::observe(run)));
    assert(exists|i: int| 0 <= i < implementation::observe(run).events.len()
        && !(implementation::observe(run).events[i] is Silent));
}
} // verus!
