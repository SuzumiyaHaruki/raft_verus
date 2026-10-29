//! Independent callers, DRAFT. Candidate types are proof adapters, not a runtime architecture.
#[path = "/home/nitro/Desktop/raft_verus/runs/fresh-system-02/runs/checks/1793270561210875663/candidate/requirements/targets.rs"]
mod targets;
#[path = "/home/nitro/Desktop/raft_verus/runs/fresh-system-02/runs/checks/1793270561210875663/candidate/requirements/support.rs"]
mod support;
#[path = "/home/nitro/Desktop/raft_verus/runs/fresh-system-02/runs/checks/1793270561210875663/candidate/madraft/proof/implementation.rs"]
mod implementation;
use vstd::prelude::*;
use implementation as c;

verus! {
pub type Execution = support::History<c::State, c::Input, c::Output, c::Message>;
pub open spec fn execution(h: Execution) -> bool {
    support::valid(h, |s| c::initial(s), |s,i,t,o| c::step(s,i,t,o), |i| c::receive(i), |o| c::emit(o))
}


pub proof fn leader_append_only(run: Execution)
    requires execution(run),
    ensures targets::leader_append_only(c::observe(run)),
{ c::prove_leader_append_only(run); }


} // verus!
