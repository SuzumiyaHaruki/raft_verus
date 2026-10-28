//! DRAFT maintenance-authored feasibility slice, enabled only by the madraft-poc feature.
//! This is the exact executable module compiled into MadSim and imported by Verus.
use crate::{state::*, io::*};
use vstd::prelude::*;
verus! {
#[derive(Debug, PartialEq, Eq)]
pub enum VoteAction {
    Persist { operation: OperationId, retry: bool },
    Reply(Message),
}
fn member(members: &Vec<u64>, id: u64) -> (yes: bool)
    ensures yes == members@.contains(id),
{
    let mut i = 0;
    while i < members.len()
        invariant i <= members.len(), forall |j: int| 0 <= j < i ==> members@[j] != id,
        decreases members.len() - i,
    {
        if members[i] == id { return true; }
        i += 1;
    }
    false
}
impl Node {
    /// Fresh storage only. Keeps the original STATE-INIT seed and its entrypoint intact.
    pub fn feasibility_follower(id: u64, members: Vec<u64>) -> (result: Result<Self, InitError>)
        ensures crate::vote_spec::constructed(result, id, members@),
    {
        if !member(&members, id) { return Err(InitError::InvalidMembership); }
        let mut i = 0;
        while i < members.len()
            invariant i <= members.len(), members@.contains(id),
                forall |a: int, b: int| 0 <= a < i && a < b < members.len() ==> members@[a] != members@[b],
            decreases members.len() - i,
        {
            let mut j = i + 1;
            while j < members.len()
                invariant i < members.len(), i < j <= members.len(),
                    forall |a: int, b: int| 0 <= a < i && a < b < members.len() ==> members@[a] != members@[b],
                    forall |b: int| i < b < j ==> members@[i as int] != members@[b],
                decreases members.len() - j,
            {
                if members[i] == members[j] { return Err(InitError::InvalidMembership); }
                j += 1;
            }
            i += 1;
        }
        proof {
            assert(members@.no_duplicates()) by {
                assert forall |a: int, b: int| 0 <= a < members.len() && 0 <= b < members.len() && a != b
                    implies members@[a] != members@[b] by { if a < b { } else { } }
            }
        }
        let n = Observation { id, members, hard: DurableState { term: 0, voted_for: None, log: Vec::new() },
            role: Role::Follower, commit: 0, delivered: 0, application_cursor: 0,
            incarnation: 0, serial: 0, deadline: 1, pending_write: None, recovery_required: false };
        proof {
            assert(crate::representation::log_view(n.hard.log@) =~= Seq::empty());
            assert(crate::initialization::initial_node(n, id, members@));
            assert(crate::vote_spec::invariant(n));
        }
        Ok(Node::from_observation(n))
    }

    /// Dedicated compatibility entrypoint; it is not the unfinished general Event dispatcher.
    /// Higher terms first persist term/reset-vote, then explicitly ask the host to redeliver.
    pub fn handle_request_vote(&mut self, m: Message) -> (r: Result<VoteAction, InputError>)
        requires old(self).wf(), crate::vote_spec::invariant(old(self).view()),
        ensures self.wf(), crate::vote_spec::invariant(self.view()),
            crate::vote_spec::result(old(self).view(), self.view(), m, r),
            crate::vote_spec::no_early_grant(r),
    {
        if self.state.recovery_required { return Err(InputError::RecoveryRequired); }
        if self.state.pending_write.is_some() { return Err(InputError::Busy); }
        if !member(&self.state.members, m.from) { return Err(InputError::UnknownPeer); }
        if m.to != self.state.id || m.from == self.state.id { return Err(InputError::MalformedMessage); }
        let (last_index, last_term) = match m.body {
            MessageBody::RequestVote { last_index, last_term } => (last_index, last_term),
            _ => return Err(InputError::MalformedMessage),
        };
        let higher = m.term > self.state.hard.term;
        let local_term = if self.state.hard.log.len() == 0 { 0 }
            else { self.state.hard.log[self.state.hard.log.len() - 1].term };
        let fresh = last_term > local_term || (last_term == local_term && last_index >= self.state.hard.log.len() as u64);
        let eligible = match self.state.hard.voted_for { None => true, Some(id) => id == m.from };
        let grant = m.term == self.state.hard.term && fresh && eligible;
        if higher || grant {
            if self.state.serial == u64::MAX { return Err(InputError::ResourceExhausted); }
            let op = OperationId { incarnation: self.state.incarnation, sequence: self.state.serial };
            self.state.serial = self.state.serial + 1;
            if higher {
                self.state.hard.term = m.term;
                self.state.hard.voted_for = None;
                self.state.role = Role::Follower;
                self.state.pending_write = Some(PendingWrite { operation: op, reply: None });
            } else {
                self.state.hard.voted_for = Some(m.from);
                let reply = Message { from: self.state.id, to: m.from, term: self.state.hard.term,
                    rpc: m.rpc, body: MessageBody::VoteResponse { granted: true } };
                self.state.pending_write = Some(PendingWrite { operation: op, reply: Some(reply) });
            }
            // SIMULATED DEFECT: release a grant before durable completion.
            if grant { return Ok(VoteAction::Reply(Message { from: self.state.id, to: m.from,
                term: self.state.hard.term, rpc: m.rpc, body: MessageBody::VoteResponse { granted: true } })); }
            Ok(VoteAction::Persist { operation: op, retry: higher })
        } else {
            Ok(VoteAction::Reply(Message { from: self.state.id, to: m.from, term: self.state.hard.term,
                rpc: m.rpc, body: MessageBody::VoteResponse { granted: false } }))
        }
    }

    /// Uses the original independent completion relation, on the same pending-write state.
    /// Kept separate from the default unfinished generation seed for this bounded experiment.
    pub fn finish_vote(&mut self, op: OperationId, kind: CompletionKind, outcome: CompletionOutcome)
        -> (r: Result<Option<Message>, InputError>)
        requires old(self).wf(), crate::vote_spec::invariant(old(self).view()),
        ensures self.wf(), crate::vote_spec::invariant(self.view()),
            crate::vote_spec::completion(old(self).view(), self.view(), op, kind, outcome, r),
    {
        if self.state.recovery_required { return Err(InputError::RecoveryRequired); }
        if op.incarnation != self.state.incarnation { return Err(InputError::StaleCompletion); }
        let p = match &self.state.pending_write { None => return Err(InputError::UnknownOperation), Some(p) => p };
        if p.operation.incarnation != op.incarnation || p.operation.sequence != op.sequence { return Err(InputError::UnknownOperation); }
        if !matches!(kind, CompletionKind::Persist) { return Err(InputError::WrongCompletionKind); }
        let pending = self.state.pending_write.take().unwrap();
        match outcome {
            CompletionOutcome::Success => Ok(pending.reply),
            CompletionOutcome::NotPerformed => { self.state.recovery_required = true; Err(InputError::StorageFailure) },
            CompletionOutcome::Unknown => { self.state.recovery_required = true; Err(InputError::RecoveryRequired) },
        }
    }
}
}
