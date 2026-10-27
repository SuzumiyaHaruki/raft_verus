//! DRAFT local projection only. Legal Raft transitions and host storage remain caller obligations.
use crate::{state::*, io::*, model, persistence_spec::*};
use vstd::prelude::*;
verus! {
pub proof fn submit_projection(before: Node, after: Node, m: model::Node,
    image: DurableState, reply: Option<Message>, token: OperationId)
    requires storage_projection(before, m), issue_error(before).is_none(),
        issue_result(before, after, image, reply, Ok(token)),
    ensures storage_projection(after, model::write(m, hard_view(image), reply_view(reply))),
{}

pub proof fn completion_projection(before: Node, after: Node, s: model::Cluster, id: model::NodeId,
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
{}
}
