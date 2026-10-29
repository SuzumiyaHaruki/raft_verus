//! Independent callers, DRAFT. Candidate types are proof adapters, not a runtime architecture.
#[path = "/home/nitro/Desktop/raft_verus/runs/fresh-system-02/runs/checks/1793269996641626505/candidate/requirements/targets.rs"]
mod targets;
#[path = "/home/nitro/Desktop/raft_verus/runs/fresh-system-02/runs/checks/1793269996641626505/candidate/requirements/support.rs"]
mod support;
#[path = "/home/nitro/Desktop/raft_verus/runs/fresh-system-02/runs/checks/1793269996641626505/candidate/madraft/proof/implementation.rs"]
mod implementation;
use vstd::prelude::*;
use implementation as c;

verus! {
pub type Execution = support::History<c::State, c::Input, c::Output, c::Message>;
pub open spec fn execution(h: Execution) -> bool {
    support::valid(h, |s| c::initial(s), |s,i,t,o| c::step(s,i,t,o), |i| c::receive(i), |o| c::emit(o))
}
// runtime interface begin
pub type RuntimeExecution = support::History<c::RuntimeState, c::Input, c::Output, c::Message>;
pub open spec fn runtime_execution(h: RuntimeExecution) -> bool {
    support::valid(h, |s| c::runtime_initial(s), |s,i,t,o| c::runtime_step(s,i,t,o), |i| c::receive(i), |o| c::emit(o))
}
pub open spec fn abstract_execution(h: RuntimeExecution) -> Execution {
    support::project(h, |s| c::view(s))
}
// runtime interface end

// Necessary witness from executable-side initialization/steps, not coverage by itself.
pub proof fn nonempty_execution()
    ensures exists|run: RuntimeExecution| runtime_execution(run)
        && targets::shape(c::observe(abstract_execution(run)))
        && exists|i: int| 0 <= i < c::observe(abstract_execution(run)).events.len()
            && !(c::observe(abstract_execution(run)).events[i] is Silent),
{
    let run = c::execution_witness();
    assert(runtime_execution(run));
    assert(targets::shape(c::observe(abstract_execution(run))));
    assert(exists|i: int| 0 <= i < c::observe(abstract_execution(run)).events.len()
        && !(c::observe(abstract_execution(run)).events[i] is Silent));
}


} // verus!
