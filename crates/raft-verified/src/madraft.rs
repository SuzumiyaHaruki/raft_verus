//! DRAFT vote API scaffold, compiled by Cargo/MadSim and imported by Verus.
//! Python-launched Codex generates the bodies and local contracts during each run.
use crate::{state::*, io::*};
use vstd::prelude::*;
verus! {
#[derive(Debug, PartialEq, Eq)]
pub enum VoteAction {
    Persist { operation: OperationId, retry: bool },
    Reply(Message),
}
impl Node {
    /// The independent vote target requires useful decisions and durable reply ordering.
    pub fn handle_request_vote(&mut self, m: Message) -> (r: Result<VoteAction, InputError>)
        requires old(self).wf(), crate::vote_spec::invariant(old(self).view()),
        ensures self.wf(),
    {
        let _ = m;
        Err(InputError::UnsupportedEvent)
    }

    /// Generate completion handling using the same node and persistence coordinator.
    pub fn finish_vote(&mut self, op: OperationId, kind: CompletionKind, outcome: CompletionOutcome)
        -> (r: Result<Option<Message>, InputError>)
        requires old(self).wf(), crate::vote_spec::invariant(old(self).view()),
        ensures self.wf(),
    {
        let _ = (op, kind, outcome);
        Err(InputError::UnsupportedEvent)
    }
}
}
