//! DRAFT local I/O contract; callers still owe legal Raft transitions and host durability.
use crate::{state::*, io::*, model};
use crate::representation::log_view;
use vstd::prelude::*;

verus! {
pub open spec fn hard_view(h: DurableState) -> model::HardState {
    model::HardState { term: h.term as nat, voted_for: match h.voted_for { Some(id) => Some(id as nat), None => None }, log: log_view(h.log@) }
}

pub open spec fn token_view(t: OperationId) -> model::Token {
    model::Token { incarnation: t.incarnation as nat, serial: t.sequence as nat }
}
pub open spec fn message_view(m: Message) -> model::Message {
    model::Message { from: m.from as nat, to: m.to as nat, term: m.term as nat, rpc: token_view(m.rpc),
        body: match m.body {
            MessageBody::RequestVote { last_index, last_term } => model::Body::RequestVote { last_index: last_index as nat, last_term: last_term as nat },
            MessageBody::VoteResponse { granted } => model::Body::VoteResponse { granted },
            MessageBody::AppendEntries { previous_index, previous_term, entries, leader_commit } => model::Body::AppendEntries { previous: previous_index as nat, previous_term: previous_term as nat, entries: log_view(entries@), commit: leader_commit as nat },
            MessageBody::AppendResponse { accepted, matched_index } => model::Body::AppendResponse { accepted, matched: matched_index as nat },
        } }
}
pub open spec fn reply_view(r: Option<Message>) -> Option<model::Message> {
    match r { Some(m) => Some(message_view(m)), None => None }
}

// A projection, not full node/cluster refinement. Disk belongs to the host.
pub open spec fn storage_projection(n: Observation, m: model::Node) -> bool {
    m.hard == hard_view(n.hard) && m.incarnation == n.incarnation
    && m.serial == n.serial && m.alive == !n.recovery_required
    && match n.pending_write {
        None => m.pending_write.is_none(),
        Some(p) => m.pending_write.is_some()
            && m.pending_write.unwrap().image == hard_view(n.hard)
            && m.pending_write.unwrap().reply == reply_view(p.reply)
            && m.pending_write.unwrap().token.incarnation == p.operation.incarnation
            && m.pending_write.unwrap().token.serial == p.operation.sequence,
    }
}

pub open spec fn issue_error(n: Observation) -> Option<InputError> {
    if n.recovery_required { Some(InputError::RecoveryRequired) }
    else if n.pending_write.is_some() { Some(InputError::Busy) }
    else if n.serial == u64::MAX { Some(InputError::ResourceExhausted) }
    else { None }
}

pub open spec fn issue_result(before: Observation, after: Observation, image: DurableState,
                             reply: Option<Message>, result: Result<OperationId, InputError>) -> bool {
    match issue_error(before) {
        Some(e) => result == Err::<OperationId, InputError>(e) && after == before,
        None => {
            let token = OperationId { incarnation: before.incarnation, sequence: before.serial };
            result == Ok::<OperationId, InputError>(token) && after == Observation { hard: image,
                pending_write: Some(PendingWrite { operation: token, reply }),
                serial: (before.serial + 1) as u64, ..before }
        },
    }
}

pub open spec fn completion_error(n: Observation, operation: OperationId, kind: CompletionKind) -> Option<InputError> {
    if n.recovery_required { Some(InputError::RecoveryRequired) }
    else if operation.incarnation != n.incarnation { Some(InputError::StaleCompletion) }
    else if n.pending_write.is_none() { Some(InputError::UnknownOperation) }
    else if n.pending_write.unwrap().operation != operation { Some(InputError::UnknownOperation) }
    else if kind != CompletionKind::Persist { Some(InputError::WrongCompletionKind) }
    else { None }
}

pub open spec fn complete_result(before: Observation, after: Observation, operation: OperationId,
    kind: CompletionKind, outcome: CompletionOutcome, result: Result<Option<Message>, InputError>) -> bool {
    match completion_error(before, operation, kind) {
        Some(e) => result == Err::<Option<Message>, InputError>(e) && after == before,
        None => match outcome {
            CompletionOutcome::Success => result == Ok::<Option<Message>, InputError>(before.pending_write.unwrap().reply)
                && after == Observation { pending_write: None, ..before },
            CompletionOutcome::NotPerformed => result == Err::<Option<Message>, InputError>(InputError::StorageFailure)
                && after == Observation { pending_write: None, recovery_required: true, ..before },
            CompletionOutcome::Unknown => result == Err::<Option<Message>, InputError>(InputError::RecoveryRequired)
                && after == Observation { pending_write: None, recovery_required: true, ..before },
        },
    }
}

pub open spec fn dispatch_result(before: Observation, after: Observation, event: Event,
                                result: Result<Option<Message>, InputError>) -> bool {
    match event {
        Event::Complete { operation, kind, outcome } => complete_result(before, after, operation, kind, outcome, result),
        _ => after == before && result == Err::<Option<Message>, InputError>(InputError::UnsupportedEvent),
    }
}
}
