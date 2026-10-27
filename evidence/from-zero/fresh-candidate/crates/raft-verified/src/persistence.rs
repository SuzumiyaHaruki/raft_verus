//! DRAFT unimplemented coordinator seed. Generate executable bodies and proofs in this file.
use crate::state::{DurableState, Node};
use crate::io::*;
#[cfg(verus_keep_ghost)]
use crate::persistence_spec::{issue_result, complete_result, dispatch_result};
use vstd::prelude::*;

verus! {
impl Node {
    /// Only a protocol handler that has established a legal transition may submit its image/reply.
    /// The host snapshots self.hard under this token before delivering any callback.
    pub fn submit_persist(&mut self, image: DurableState, reply: Option<Message>) -> (result: Result<OperationId, InputError>)
        ensures issue_result(*old(self), *self, image, reply, result),
    {
        // Intentionally incomplete: this error does not satisfy the contract.
        let _ = image;
        let _ = reply;
        Err(InputError::UnsupportedEvent)
    }

    pub fn complete_persist(&mut self, operation: OperationId, kind: CompletionKind, outcome: CompletionOutcome)
        -> (result: Result<Option<Message>, InputError>)
        ensures complete_result(*old(self), *self, operation, kind, outcome, result),
    {
        // Intentionally incomplete: this error does not satisfy the contract.
        let _ = operation;
        let _ = kind;
        let _ = outcome;
        Err(InputError::UnsupportedEvent)
    }

    /// Partial dispatcher shared by host drivers. Unsupported events never fake success.
    pub fn step(&mut self, event: Event) -> (result: Result<Option<Message>, InputError>)
        ensures dispatch_result(*old(self), *self, event, result),
    {
        // Intentionally incomplete: this error does not satisfy the contract.
        let _ = event;
        Err(InputError::UnsupportedEvent)
    }
}
}
