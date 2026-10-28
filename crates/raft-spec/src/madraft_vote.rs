//! DRAFT additive feasibility targets. Existing model and dispatcher targets are unchanged.
use crate::{state::*, io::*, madraft::VoteAction, initialization, persistence_spec};
use vstd::prelude::*;
verus! {
pub open spec fn invariant(n: Observation) -> bool {
    initialization::valid_membership(n.id, n.members@)
    && (n.hard.voted_for.is_some() ==> n.members@.contains(n.hard.voted_for.unwrap()))
    && match n.pending_write {
        None => true,
        Some(p) => p.operation.incarnation == n.incarnation && p.operation.sequence < n.serial
            && match p.reply {
                None => true,
                Some(r) => r.from == n.id && n.members@.contains(r.to)
                    && r.term == n.hard.term && r.body == MessageBody::VoteResponse { granted: true }
                    && n.hard.voted_for == Some(r.to),
            },
    }
}
pub open spec fn constructed(result: Result<Node, InitError>, id: u64, members: Seq<u64>) -> bool {
    match result {
        Ok(n) => n.wf() && invariant(n.view()) && initialization::init_result(Ok(n.view()), id, members),
        Err(e) => initialization::init_result(Err(e), id, members),
    }
}
pub open spec fn fresh(n: Observation, last_index: u64, last_term: u64) -> bool {
    let term = if n.hard.log@.len() == 0 { 0 } else { n.hard.log@[n.hard.log@.len() - 1].term };
    last_term > term || (last_term == term && last_index >= n.hard.log@.len())
}
pub open spec fn input_error(n: Observation, m: Message) -> Option<InputError> {
    if n.recovery_required { Some(InputError::RecoveryRequired) }
    else if n.pending_write.is_some() { Some(InputError::Busy) }
    else if !n.members@.contains(m.from) { Some(InputError::UnknownPeer) }
    else if m.to != n.id || m.from == n.id { Some(InputError::MalformedMessage) }
    else { match m.body { MessageBody::RequestVote { .. } => None, _ => Some(InputError::MalformedMessage) } }
}
pub open spec fn response(n: Observation, m: Message, granted: bool) -> Message {
    Message { from: n.id, to: m.from, term: n.hard.term, rpc: m.rpc, body: MessageBody::VoteResponse { granted } }
}
pub open spec fn result(before: Observation, after: Observation, m: Message,
    result: Result<VoteAction, InputError>) -> bool {
    match input_error(before, m) {
        Some(e) => after == before && result == Err::<VoteAction, InputError>(e),
        None => {
            let higher = m.term > before.hard.term;
            let grant = m.term == before.hard.term
                && fresh(before, m.body->RequestVote_last_index, m.body->RequestVote_last_term)
                && (before.hard.voted_for.is_none() || before.hard.voted_for == Some(m.from));
            if higher || grant {
                if before.serial == u64::MAX {
                    after == before && result == Err::<VoteAction, InputError>(InputError::ResourceExhausted)
                } else {
                    let op = OperationId { incarnation: before.incarnation, sequence: before.serial };
                    let hard = if higher { DurableState { term: m.term, voted_for: None, ..before.hard } }
                        else { DurableState { voted_for: Some(m.from), ..before.hard } };
                    let reply = if higher { None } else { Some(response(before, m, true)) };
                    after == Observation { hard, role: if higher { Role::Follower } else { before.role },
                        serial: (before.serial + 1) as u64, pending_write: Some(PendingWrite { operation: op, reply }), ..before }
                    && result == Ok::<VoteAction, InputError>(VoteAction::Persist { operation: op, retry: higher })
                }
            } else {
                after == before && result == Ok::<VoteAction, InputError>(VoteAction::Reply(response(before, m, false)))
            }
        },
    }
}
// Explicit barrier: receive never returns a grant; release is a separate matching completion.
pub open spec fn no_early_grant(r: Result<VoteAction, InputError>) -> bool {
    match r { Ok(VoteAction::Reply(m)) => m.body == MessageBody::VoteResponse { granted: false }, _ => true }
}
pub open spec fn completion(before: Observation, after: Observation, op: OperationId,
    kind: CompletionKind, outcome: CompletionOutcome, r: Result<Option<Message>, InputError>) -> bool {
    persistence_spec::complete_result(before, after, op, kind, outcome, r)
}
}
