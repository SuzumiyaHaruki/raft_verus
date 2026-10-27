//! DRAFT shared event and completion vocabulary; same types in Cargo and Verus.
use crate::reconcile::{Entry, RequestId};
use crate::state::DurableState;
use vstd::prelude::*;
type NodeId = u64;
type Term = u64;
type LogIndex = u64;
verus! {
/// The host supplies a fresh incarnation on recovery; never reuse a live token.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OperationId {
    pub incarnation: u64,
    pub sequence: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MessageBody {
    RequestVote {
        last_index: LogIndex,
        last_term: Term,
    },
    VoteResponse {
        granted: bool,
    },
    AppendEntries {
        previous_index: LogIndex,
        previous_term: Term,
        entries: Vec<Entry>,
        leader_commit: LogIndex,
    },
    AppendResponse {
        accepted: bool,
        matched_index: LogIndex,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Message {
    pub from: NodeId,
    pub to: NodeId,
    pub term: Term,
    pub rpc: OperationId,
    pub body: MessageBody,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompletionKind {
    Persist,
    Send,
    Apply,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompletionOutcome {
    Success,
    /// The host certifies the effect did not occur; retry may use a new token.
    NotPerformed,
    /// Whether the effect occurred is unknown; persistence requires recovery.
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Event {
    Receive(Message),
    ElectionTimeout {
        incarnation: u64,
        generation: u64,
    },
    HeartbeatTimeout {
        incarnation: u64,
        generation: u64,
    },
    Propose {
        request: RequestId,
        bytes: Vec<u8>,
    },
    Complete {
        operation: OperationId,
        kind: CompletionKind,
        outcome: CompletionOutcome,
    },
    Recover {
        incarnation: u64,
        durable: DurableState,
        application_cursor: LogIndex,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Effect {
    Send {
        operation: OperationId,
        message: Message,
    },
    Persist {
        operation: OperationId,
        state: DurableState,
    },
    Deliver {
        operation: OperationId,
        index: LogIndex,
        entry: Entry,
    },
    ResetElectionTimer {
        incarnation: u64,
        generation: u64,
    },
    ResetHeartbeatTimer {
        incarnation: u64,
        generation: u64,
    },
    CancelTimer {
        incarnation: u64,
        generation: u64,
    },
    ProposalAccepted {
        request: RequestId,
        index: LogIndex,
        term: Term,
    },
    Reject {
        request: Option<RequestId>,
        error: InputError,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputError {
    UnknownPeer,
    MalformedMessage,
    InvalidIndex,
    StaleCompletion,
    UnknownOperation,
    WrongCompletionKind,
    NotLeader,
    Busy,
    ResourceExhausted,
    RecoveryRequired,
    StorageFailure,
    UnsupportedEvent,
}

}
