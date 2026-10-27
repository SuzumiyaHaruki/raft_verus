//! DRAFT one-write coordinator on the shared Node. No election/replication legality claim.
use crate::state::{DurableState, Node, PendingWrite};
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
        if self.recovery_required { return Err(InputError::RecoveryRequired); }
        if self.pending_write.is_some() { return Err(InputError::Busy); }
        if self.serial == u64::MAX { return Err(InputError::ResourceExhausted); }
        let operation = OperationId { incarnation: self.incarnation, sequence: self.serial };
        self.serial = self.serial + 1;
        self.hard = image;
        self.pending_write = Some(PendingWrite { operation, reply });
        Ok(operation)
    }

    pub fn complete_persist(&mut self, operation: OperationId, kind: CompletionKind, outcome: CompletionOutcome)
        -> (result: Result<Option<Message>, InputError>)
        ensures complete_result(*old(self), *self, operation, kind, outcome, result),
    {
        if self.recovery_required { return Err(InputError::RecoveryRequired); }
        if operation.incarnation != self.incarnation { return Err(InputError::StaleCompletion); }
        let pending = match &self.pending_write {
            Some(p) => p,
            None => { return Err(InputError::UnknownOperation); }
        };
        if operation.incarnation != pending.operation.incarnation || operation.sequence != pending.operation.sequence {
            return Err(InputError::UnknownOperation);
        }
        if !matches!(kind, CompletionKind::Persist) { return Err(InputError::WrongCompletionKind); }
        let pending = self.pending_write.take().unwrap();
        match outcome {
            CompletionOutcome::Success => Ok(pending.reply),
            CompletionOutcome::NotPerformed => {
                self.recovery_required = true;
                Err(InputError::StorageFailure)
            },
            CompletionOutcome::Unknown => {
                self.recovery_required = true;
                Err(InputError::RecoveryRequired)
            },
        }
    }

    /// Partial dispatcher shared by host drivers. Unsupported events never fake success.
    pub fn step(&mut self, event: Event) -> (result: Result<Option<Message>, InputError>)
        ensures dispatch_result(*old(self), *self, event, result),
    {
        match event {
            Event::Complete { operation, kind, outcome } => self.complete_persist(operation, kind, outcome),
            _ => Err(InputError::UnsupportedEvent),
        }
    }
}
}
