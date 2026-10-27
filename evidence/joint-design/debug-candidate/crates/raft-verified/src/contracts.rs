//! SIMULATED generated implementation contracts; independent targets remain unchanged.
use crate::{state::*, io::*, initialization, persistence_spec};
use vstd::prelude::*;
verus! {
pub open spec fn constructed(result: Result<Node, InitError>, id: u64, members: Seq<u64>) -> bool {
    match result {
        Ok(n) => n.wf() && initialization::init_result(Ok(n.view()), id, members),
        Err(e) => initialization::init_result(Err(e), id, members),
    }
}
pub open spec fn issue_result(before: Node, after: Node, image: DurableState,
    reply: Option<Message>, result: Result<OperationId, InputError>) -> bool {
    after.wf() && persistence_spec::issue_result(before.view(), after.view(), image, reply, result)
}
pub open spec fn complete_result(before: Node, after: Node, operation: OperationId,
    kind: CompletionKind, outcome: CompletionOutcome, result: Result<Option<Message>, InputError>) -> bool {
    after.wf() && persistence_spec::complete_result(before.view(), after.view(), operation, kind, outcome, result)
}
pub open spec fn dispatch_result(before: Node, after: Node, event: Event,
    result: Result<Option<Message>, InputError>) -> bool {
    after.wf() && persistence_spec::dispatch_result(before.view(), after.view(), event, result)
}
}
