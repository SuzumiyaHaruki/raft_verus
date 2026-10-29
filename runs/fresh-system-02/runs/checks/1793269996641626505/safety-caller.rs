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
// obligation: election_safety
pub proof fn election_safety(run: Execution)
    requires execution(run),
    ensures targets::election_safety(c::observe(run)),
{ c::prove_election_safety(run); }

// obligation: leader_append_only
pub proof fn leader_append_only(run: Execution)
    requires execution(run),
    ensures targets::leader_append_only(c::observe(run)),
{ c::prove_leader_append_only(run); }

// obligation: log_matching
pub proof fn log_matching(run: Execution)
    requires execution(run),
    ensures targets::log_matching(c::observe(run)),
{ c::prove_log_matching(run); }

// obligation: leader_completeness
pub proof fn leader_completeness(run: Execution)
    requires execution(run),
    ensures targets::leader_completeness(c::observe(run)),
{ c::prove_leader_completeness(run); }

// obligation: state_machine_safety
pub proof fn state_machine_safety(run: Execution)
    requires execution(run),
    ensures targets::state_machine_safety(c::observe(run)),
{ c::prove_state_machine_safety(run); }

// obligation: execution_link
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

// obligation: snapshot_observation
pub proof fn snapshot_and_observation_shape(run: Execution)
    requires execution(run),
    ensures targets::shape(c::observe(run)),
        targets::snapshot_fidelity(c::observe(run), c::applications()),
{ c::prove_observation_and_snapshots(run); }

// obligation: nonempty_execution
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

// obligation: composed_safety
pub proof fn actual_execution_safety(run: RuntimeExecution)
    requires runtime_execution(run),
    ensures targets::election_safety(c::observe(abstract_execution(run))),
        targets::leader_append_only(c::observe(abstract_execution(run))),
        targets::log_matching(c::observe(abstract_execution(run))),
        targets::leader_completeness(c::observe(abstract_execution(run))),
        targets::state_machine_safety(c::observe(abstract_execution(run))),
        targets::shape(c::observe(abstract_execution(run))),
        targets::snapshot_fidelity(c::observe(abstract_execution(run)), c::applications()),
{
    execution_link(run);
    let h = abstract_execution(run);
    election_safety(h); leader_append_only(h); log_matching(h);
    leader_completeness(h); state_machine_safety(h);
    snapshot_and_observation_shape(h);
}
} // verus!
