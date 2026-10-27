//! DRAFT small trace-level progress target. No Raft leadership/commitment theorem.
use crate::{state::Node, io::*, persistence_spec::*};
use vstd::prelude::*;
verus! {
pub open spec fn successful(e: Event, token: OperationId) -> bool {
    match e { Event::Complete { operation, kind, outcome } => operation == token
        && kind == CompletionKind::Persist && outcome == CompletionOutcome::Success, _ => false }
}
pub open spec fn waiting_event(n: Node, e: Event) -> bool {
    match e { Event::Complete { operation, kind, .. } => completion_error(n, operation, kind).is_some(), _ => true }
}
// Host condition: a matching successful callback gets service after finitely many
// rejected/stale/unsupported events. This mentions inputs, never eventual outputs.
pub open spec fn fair_completion(n: Node, events: spec_fn(nat) -> Event, start: nat, token: OperationId) -> bool {
    exists |k: nat| k >= start && #[trigger] successful(events(k), token)
        && forall |i: nat| start <= i < k ==> #[trigger] waiting_event(n, events(i))
}
pub open spec fn execution(nodes: spec_fn(nat) -> Node, events: spec_fn(nat) -> Event,
    outputs: spec_fn(nat) -> Result<Option<Message>, InputError>, start: nat) -> bool {
    forall |i: nat| i >= start ==> #[trigger] dispatch_result(nodes(i), nodes(i + 1), events(i), outputs(i))
}
pub open spec fn ready(n: Node, token: OperationId) -> bool {
    completion_error(n, token, CompletionKind::Persist).is_none()
}
pub open spec fn eventually_released(nodes: spec_fn(nat) -> Node,
    outputs: spec_fn(nat) -> Result<Option<Message>, InputError>, start: nat) -> bool {
    exists |k: nat| k >= start && #[trigger] outputs(k) == Ok::<Option<Message>, InputError>(nodes(start).pending_write.unwrap().reply)
        && nodes(k + 1).pending_write.is_none()
}
}
