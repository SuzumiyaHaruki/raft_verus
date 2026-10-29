//! Independent callers, DRAFT. Candidate types are proof adapters, not a runtime architecture.
#[path = "/home/nitro/Desktop/raft_verus/runs/fresh-system-02/runs/checks/1793271014387829434/candidate/requirements/targets.rs"]
mod targets;
#[path = "/home/nitro/Desktop/raft_verus/runs/fresh-system-02/runs/checks/1793271014387829434/candidate/requirements/support.rs"]
mod support;
#[path = "/home/nitro/Desktop/raft_verus/runs/fresh-system-02/runs/checks/1793271014387829434/candidate/madraft/proof/implementation.rs"]
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

pub proof fn execution_link(run: RuntimeExecution)
    requires runtime_execution(run),
    ensures execution(abstract_execution(run)),
        abstract_execution(run).inputs == run.inputs,
        abstract_execution(run).outputs == run.outputs,
        abstract_execution(run).sent == run.sent,
{
    assert forall|s: c::RuntimeState| c::runtime_initial(s) implies
        #[trigger] c::runtime_invariant(s, Seq::empty()) && c::initial(c::view(s)) by {
        c::prove_runtime_initial(s);
    }
    assert forall|s: c::RuntimeState, i: c::Input, t: c::RuntimeState, o: c::Output,
        a: Seq<support::Envelope<c::Message>>, b: Seq<support::Envelope<c::Message>>|
        #[trigger] c::runtime_invariant(s,a) && #[trigger] c::runtime_step(s,i,t,o)
        && #[trigger] support::exchange(a,c::receive(i),c::emit(o),b)
        implies c::runtime_invariant(t,b) && c::step(c::view(s),i,c::view(t),o) by {
        c::prove_runtime_step(s,i,t,o,a,b);
    }
    support::simulate(run, |s| c::runtime_initial(s), |s,i,t,o| c::runtime_step(s,i,t,o),
        |s| c::initial(s), |s,i,t,o| c::step(s,i,t,o), |i| c::receive(i), |o| c::emit(o),
        |s| c::view(s), |s,sent| c::runtime_invariant(s,sent));
}


} // verus!
