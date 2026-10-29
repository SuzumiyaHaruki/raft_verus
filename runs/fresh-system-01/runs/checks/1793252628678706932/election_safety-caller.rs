//! Independent callers, DRAFT. The generator must create implementation.rs.
//! Its execution predicate may contain only initialization and actual permitted
//! transitions; unproved reachable-state invariants are NOT allowed premises.
#[path = "/home/nitro/Desktop/raft_verus/runs/fresh-system-01/runs/checks/1793252628678706932/candidate/requirements/targets.rs"]
mod targets;
#[path = "/home/nitro/Desktop/raft_verus/runs/fresh-system-01/runs/checks/1793252628678706932/candidate/madraft/proof/implementation.rs"]
mod implementation;
use vstd::prelude::*;

verus! {

pub proof fn election_safety(run: implementation::Execution)
    requires implementation::execution(run),
    ensures targets::election_safety(implementation::observe(run)),
{ implementation::prove_election_safety(run); }


} // verus!
