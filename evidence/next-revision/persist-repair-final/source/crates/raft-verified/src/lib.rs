//! DRAFT Raft interface and verified local log reconciliation.
//! No complete Raft node or `step` implementation is provided.
//! `u64` exhaustion must return an error; the unbounded model is not a resource proof.
#![forbid(unsafe_code)]

pub type NodeId = u64;
pub type Term = u64;
pub type LogIndex = u64;

pub mod reconcile;
pub use reconcile::{reconcile_log, Command, Entry, ReconcileError, RequestId};

pub mod state;
pub mod node;
pub use state::{DurableState, InitError, Node, Role};

pub mod io;
pub mod persistence;
pub use io::*;

/// Small runnable interface helper only; this is not a protocol handler.
#[derive(Debug)]
pub struct OperationSequence {
    incarnation: u64,
    next: Option<u64>,
}

impl OperationSequence {
    pub fn new(incarnation: u64) -> Self {
        Self {
            incarnation,
            next: Some(0),
        }
    }

    pub fn allocate(&mut self) -> Result<OperationId, InputError> {
        let sequence = self.next.ok_or(InputError::ResourceExhausted)?;
        self.next = sequence.checked_add(1);
        Ok(OperationId {
            incarnation: self.incarnation,
            sequence,
        })
    }

    /// Match the full token and kind; duplicates become stale after removal.
    pub fn validate_completion(
        expected: Option<(OperationId, CompletionKind)>,
        received: OperationId,
        kind: CompletionKind,
    ) -> Result<(), InputError> {
        let (token, expected_kind) = expected.ok_or(InputError::UnknownOperation)?;
        if received != token {
            return Err(InputError::StaleCompletion);
        }
        if kind != expected_kind {
            return Err(InputError::WrongCompletionKind);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completions_distinguish_incarnation_kind_and_duplicate() {
        let mut seq = OperationSequence::new(7);
        let token = seq.allocate().unwrap();
        assert_eq!(seq.allocate().unwrap().sequence, 1);
        let expected = Some((token, CompletionKind::Persist));
        assert_eq!(
            OperationSequence::validate_completion(expected, token, CompletionKind::Persist),
            Ok(())
        );
        assert_eq!(
            OperationSequence::validate_completion(
                expected,
                OperationId {
                    incarnation: 6,
                    ..token
                },
                CompletionKind::Persist
            ),
            Err(InputError::StaleCompletion)
        );
        assert_eq!(
            OperationSequence::validate_completion(expected, token, CompletionKind::Apply),
            Err(InputError::WrongCompletionKind)
        );
        assert_eq!(
            OperationSequence::validate_completion(None, token, CompletionKind::Persist),
            Err(InputError::UnknownOperation)
        );
    }

    #[test]
    fn exhaustion_never_reuses_a_token() {
        let mut seq = OperationSequence {
            incarnation: 1,
            next: Some(u64::MAX),
        };
        assert_eq!(seq.allocate().unwrap().sequence, u64::MAX);
        assert_eq!(seq.allocate(), Err(InputError::ResourceExhausted));
        assert_eq!(seq.allocate(), Err(InputError::ResourceExhausted));
    }
}
