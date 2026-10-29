//! Independent callers, DRAFT. Candidate types are proof adapters, not a runtime architecture.
#[path = "/home/nitro/Desktop/raft_verus/runs/fresh-system-02/runs/checks/1793271554216407747/candidate/requirements/targets.rs"]
mod targets;
#[path = "/home/nitro/Desktop/raft_verus/runs/fresh-system-02/runs/checks/1793271554216407747/candidate/requirements/support.rs"]
mod support;
#[path = "/home/nitro/Desktop/raft_verus/runs/fresh-system-02/runs/checks/1793271554216407747/candidate/madraft/proof/implementation.rs"]
mod implementation;
use vstd::prelude::*;
use implementation as c;

verus! {
pub type Execution = support::History<c::State, c::Input, c::Output, c::Message>;
pub open spec fn execution(h: Execution) -> bool {
    support::valid(h, |s| c::initial(s), |s,i,t,o| c::step(s,i,t,o), |i| c::receive(i), |o| c::emit(o))
}


pub proof fn election_safety(run: Execution)
    requires execution(run),
    ensures targets::election_safety(c::observe(run)),
{ c::prove_election_safety(run); }


} // verus!
