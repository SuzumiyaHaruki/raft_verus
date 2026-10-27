//! DRAFT independent early progress consumer over the actual dispatcher contract.
use vstd::prelude::*;
#[path = "../raft-verified/src/node_state.rs"] pub mod node_state;
#[path = "../raft-verified/src/contracts.rs"] pub mod contracts;
#[path = "observation.rs"] pub mod observation;
#[path = "../raft-spec/src/lib.rs"] pub mod model;
#[path = "../raft-verified/src/types.rs"] pub mod types;
#[path = "../raft-spec/src/reconcile_view.rs"] pub mod representation;
#[path = "../raft-verified/src/state.rs"] pub mod state;
#[path = "../raft-verified/src/io.rs"] pub mod io;
#[path = "../raft-spec/src/persistence.rs"] pub mod persistence_spec;
#[path = "../raft-spec/src/node_init.rs"] pub mod initialization;
#[path = "../raft-verified/src/persistence.rs"] pub mod persistence;
#[path = "../raft-spec/src/progress.rs"] pub mod progress_spec;
#[path = "progress_lemmas.rs"] pub mod progress_lemmas;
verus! {
pub proof fn check_local_progress(nodes: spec_fn(nat) -> state::Observation, events: spec_fn(nat) -> io::Event,
    outputs: spec_fn(nat) -> Result<Option<io::Message>, io::InputError>, start: nat, token: io::OperationId)
    requires progress_spec::ready(nodes(start), token), progress_spec::execution(nodes, events, outputs, start),
        progress_spec::fair_completion(nodes(start), events, start, token),
    ensures progress_spec::eventually_released(nodes, outputs, start),
{ progress_lemmas::release_after_service(nodes, events, outputs, start, token); }

pub fn check_step(n: &mut state::Node, event: io::Event) -> (result: Result<Option<io::Message>, io::InputError>)
    requires old(n).wf(),
    ensures n.wf(), persistence_spec::dispatch_result(old(n).view(), n.view(), event, result),
{ n.step(event) }
}
