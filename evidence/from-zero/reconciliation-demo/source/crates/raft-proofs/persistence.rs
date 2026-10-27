//! DRAFT exact executable consumers; no strengthened preconditions or weakened targets.
use vstd::prelude::*;
use crate::{state::*, io::*, persistence_spec::*};
#[path = "../raft-spec/src/lib.rs"] pub mod model;
#[path = "../raft-verified/src/types.rs"] pub mod types;
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
pub proof fn check_completion_projection(before: Node, after: Node, s: model::Cluster, id: model::NodeId,
    token: OperationId, result: Result<Option<Message>, InputError>)
    requires s.members.contains(id), s.nodes.contains_key(id), storage_projection(before, s.nodes[id]),
        completion_error(before, token, CompletionKind::Persist).is_none(),
        s.nodes[id].disk == s.nodes[id].hard,
        complete_result(before, after, token, CompletionKind::Persist, CompletionOutcome::Success, result),
    ensures ({
        let m = s.nodes[id];
        let next = model::Node { pending_write: None,
            outbox: match m.pending_write.unwrap().reply { Some(message) => m.outbox.push(message), None => m.outbox }, ..m };
        storage_projection(after, next)
        && model::storage_ack(s, model::set_node(s, id, next), id, token_view(token))
        && result.is_ok() && reply_view(result.unwrap()) == m.pending_write.unwrap().reply
    }),
{ persistence_lemmas::completion_projection(before, after, s, id, token, result); }
}
