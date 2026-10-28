//! DRAFT independent consumers of the exact Cargo/MadSim executable source.
use vstd::prelude::*;
#[path = "../raft-verified/src/types.rs"] pub mod types;
#[path = "../raft-verified/src/state.rs"] pub mod state;
#[path = "../raft-verified/src/node_state.rs"] pub mod node_state;
#[path = "../raft-verified/src/node.rs"] pub mod node;
#[path = "../raft-verified/src/contracts.rs"] pub mod contracts;
#[path = "../raft-verified/src/persistence.rs"] pub mod persistence;
#[path = "../raft-verified/src/io.rs"] pub mod io;
#[path = "../raft-spec/src/lib.rs"] pub mod model;
#[path = "../raft-spec/src/reconcile_view.rs"] pub mod representation;
#[path = "../raft-spec/src/node_init.rs"] pub mod initialization;
#[path = "../raft-spec/src/persistence.rs"] pub mod persistence_spec;
#[path = "../raft-spec/src/madraft_vote.rs"] pub mod vote_spec;
#[path = "../raft-verified/src/madraft.rs"] pub mod madraft;
#[path = "observation.rs"] pub mod observation;
use state::*;
use io::*;
verus! {
pub fn check_follower(id: u64, members: Vec<u64>) -> (r: Result<Node, InitError>)
    ensures vote_spec::constructed(r, id, members@),
{ Node::new(id, members) }

pub fn check_vote(n: &mut Node, m: Message) -> (r: Result<madraft::VoteAction, InputError>)
    requires old(n).wf(), vote_spec::invariant(old(n).view()),
    ensures n.wf(), vote_spec::invariant(n.view()), vote_spec::result(old(n).view(), n.view(), m, r),
        vote_spec::no_early_grant(r),
{ n.handle_request_vote(m) }

pub fn check_durable_reply(n: &mut Node, op: OperationId, kind: CompletionKind, outcome: CompletionOutcome)
    -> (r: Result<Option<Message>, InputError>)
    requires old(n).wf(), vote_spec::invariant(old(n).view()),
    ensures n.wf(), vote_spec::invariant(n.view()),
        persistence_spec::complete_result(old(n).view(), n.view(), op, kind, outcome, r),
        r.is_ok() && r.unwrap().is_some() ==> outcome == CompletionOutcome::Success
            && kind == CompletionKind::Persist && old(n).view().pending_write.is_some()
            && old(n).view().pending_write.unwrap().operation == op
            && r.unwrap().unwrap().term == n.view().hard.term
            && n.view().hard.voted_for == Some(r.unwrap().unwrap().to),
{ n.finish_vote(op, kind, outcome) }

// Same log freshness comparison as receive_vote_request, through the existing entry view.
pub proof fn check_freshness_projection(n: Observation, index: u64, term: u64)
    ensures vote_spec::fresh(n, index, term) == ({
        let log = representation::log_view(n.hard.log@);
        term > model::term_at(log, log.len())
            || (term == model::term_at(log, log.len()) && index >= log.len())
    }),
{ }
}
