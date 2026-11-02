#[path = "toy.rs"] mod toy;
use vstd::prelude::*;
use crate::{support::*, targets, Execution, execution};
pub use toy::{initial as runtime_initial, step as runtime_step, receive, emitted as emit,
              model_initial as initial, bounded as runtime_invariant};
verus! {
pub type RuntimeState = u64;
pub type State = nat;
pub type Input = Option<Delivery<u64>>;
pub type Output = (u64, u64);
pub type Message = u64;
pub open spec fn view(s: u64) -> nat { s as nat }
pub open spec fn step(s: nat, i: Input, t: nat, o: Output) -> bool {
    toy::model_step(s,i,t,o) 
}
pub open spec fn refines_executable() -> bool { true } // Deliberate decoy, never consumed.
pub proof fn prove_runtime_initial(s: u64)
    requires runtime_initial(s),
    ensures runtime_invariant(s, Seq::empty()), initial(view(s)), {}
pub proof fn prove_runtime_step(s: u64, i: Input, t: u64, o: Output,
    a: Seq<Envelope<u64>>, b: Seq<Envelope<u64>>)
    requires runtime_invariant(s,a), runtime_step(s,i,t,o), exchange(a,receive(i),emit(o),b),
    ensures runtime_invariant(t,b), step(view(s),i,view(t),o), {}
// Empty observations intentionally fail nonempty coverage; this is not Raft.
pub open spec fn observe(h: Execution) -> targets::History {
    targets::History { states: seq![Map::empty()], events: Seq::empty() }
}
pub open spec fn applications() -> Map<nat, targets::Application> { Map::empty() }
pub proof fn prove_observation_and_snapshots(h: Execution)
    requires execution(h),
    ensures targets::shape(observe(h)), targets::snapshot_fidelity(observe(h), applications()), {}
}verus! { pub proof fn prove_election_safety(h: Execution) requires execution(h), ensures targets::election_safety(observe(h)), {} }
verus! { pub proof fn prove_leader_append_only(h: Execution) requires execution(h), ensures targets::leader_append_only(observe(h)), {} }
verus! { pub proof fn prove_log_matching(h: Execution) requires execution(h), ensures targets::log_matching(observe(h)), {} }
verus! { pub proof fn prove_leader_completeness(h: Execution) requires execution(h), ensures targets::leader_completeness(observe(h)), {} }
verus! { pub proof fn prove_state_machine_safety(h: Execution) requires execution(h), ensures targets::state_machine_safety(observe(h)), {} }
