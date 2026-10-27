//! DRAFT implementation-layer contract seeds. Generate predicates and prove the independent bridge.
//! These deliberately vacuous placeholders cannot satisfy the protected consumers.
use crate::{state::*, io::*};
use vstd::prelude::*;

verus! {
pub open spec fn constructed(result: Result<Node, InitError>, id: u64, members: Seq<u64>) -> bool { true }
pub open spec fn issue_result(before: Node, after: Node, image: DurableState,
    reply: Option<Message>, result: Result<OperationId, InputError>) -> bool { true }
pub open spec fn complete_result(before: Node, after: Node, operation: OperationId,
    kind: CompletionKind, outcome: CompletionOutcome, result: Result<Option<Message>, InputError>) -> bool { true }
pub open spec fn dispatch_result(before: Node, after: Node, event: Event,
    result: Result<Option<Message>, InputError>) -> bool { true }
}
