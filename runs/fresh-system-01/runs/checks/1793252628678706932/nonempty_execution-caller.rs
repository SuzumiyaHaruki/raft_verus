//! Independent callers, DRAFT. The generator must create implementation.rs.
//! Its execution predicate may contain only initialization and actual permitted
//! transitions; unproved reachable-state invariants are NOT allowed premises.
#[path = "/home/nitro/Desktop/raft_verus/runs/fresh-system-01/runs/checks/1793252628678706932/candidate/requirements/targets.rs"]
mod targets;
#[path = "/home/nitro/Desktop/raft_verus/runs/fresh-system-01/runs/checks/1793252628678706932/candidate/madraft/proof/implementation.rs"]
mod implementation;
use vstd::prelude::*;

verus! {

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
