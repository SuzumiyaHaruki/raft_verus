//! Independent callers, DRAFT. The generator must create implementation.rs.
//! Its execution predicate may contain only initialization and actual permitted
//! transitions; unproved reachable-state invariants are NOT allowed premises.
#[path = "/home/nitro/Desktop/raft_verus/runs/fresh-system-01/runs/checks/1793253254086419373/candidate/requirements/targets.rs"]
mod targets;
#[path = "/home/nitro/Desktop/raft_verus/runs/fresh-system-01/runs/checks/1793253254086419373/candidate/madraft/proof/implementation.rs"]
mod implementation;
use vstd::prelude::*;

verus! {

pub proof fn log_matching(run: implementation::Execution)
    requires implementation::execution(run),
    ensures targets::log_matching(implementation::observe(run)),
{ implementation::prove_log_matching(run); }


} // verus!
