//! DRAFT exact executable consumers; no strengthened preconditions or weakened targets.
use vstd::prelude::*;
#[path = "../raft-spec/src/lib.rs"] pub mod model;
#[path = "../raft-verified/src/reconcile.rs"] pub mod reconcile;
#[path = "../raft-spec/src/reconcile_view.rs"] pub mod representation;
#[path = "../raft-verified/src/state.rs"] pub mod state;
#[path = "../raft-verified/src/io.rs"] pub mod io;
#[path = "../raft-spec/src/persistence.rs"] pub mod persistence_spec;
#[path = "../raft-verified/src/persistence.rs"] pub mod persistence;
#[path = "persistence_lemmas.rs"] pub mod persistence_lemmas;

verus! {
pub fn check_submit(n: &mut state::Node, image: state::DurableState, reply: Option<io::Message>)
    -> (result: Result<io::OperationId, io::InputError>)
    ensures persistence_spec::issue_result(*old(n), *n, image, reply, result),
{ n.submit_persist(image, reply) }

pub fn check_complete(n: &mut state::Node, operation: io::OperationId, kind: io::CompletionKind, outcome: io::CompletionOutcome)
    -> (result: Result<Option<io::Message>, io::InputError>)
    ensures persistence_spec::complete_result(*old(n), *n, operation, kind, outcome, result),
{ n.complete_persist(operation, kind, outcome) }

pub fn check_dispatch(n: &mut state::Node, event: io::Event) -> (result: Result<Option<io::Message>, io::InputError>)
    ensures persistence_spec::dispatch_result(*old(n), *n, event, result),
{ n.step(event) }

pub proof fn check_storage_projection(before: state::Node, after: state::Node, m: model::Node,
    image: state::DurableState, reply: Option<io::Message>, token: io::OperationId)
    requires persistence_spec::storage_projection(before, m), persistence_spec::issue_error(before).is_none(),
        persistence_spec::issue_result(before, after, image, reply, Ok(token)),
    ensures persistence_spec::storage_projection(after, model::write(m, persistence_spec::hard_view(image), persistence_spec::reply_view(reply))),
{ persistence_lemmas::submit_projection(before, after, m, image, reply, token); }
}
