//! DRAFT independent caller for the generated public event path, not global Raft safety.
use vstd::prelude::*;
#[path = "../raft-verified/src/contracts.rs"]
pub mod contracts;
#[path = "../raft-spec/src/node_init.rs"]
pub mod initialization;
#[path = "../raft-verified/src/io.rs"]
pub mod io;
#[path = "../raft-verified/src/madraft.rs"]
pub mod madraft;
#[path = "../raft-spec/src/lib.rs"]
pub mod model;
#[path = "../raft-verified/src/node.rs"]
pub mod node;
#[path = "../raft-verified/src/node_state.rs"]
pub mod node_state;
#[path = "observation.rs"]
pub mod observation;
#[path = "../raft-verified/src/persistence.rs"]
pub mod persistence;
#[path = "../raft-spec/src/persistence.rs"]
pub mod persistence_spec;
#[path = "../raft-verified/src/protocol.rs"]
pub mod protocol;
#[path = "../raft-verified/src/reconcile.rs"]
pub mod reconcile;
#[path = "../raft-spec/src/reconcile_view.rs"]
pub mod representation;
#[path = "../raft-verified/src/state.rs"]
pub mod state;
#[path = "../raft-verified/src/types.rs"]
pub mod types;
#[path = "../raft-spec/src/madraft_vote.rs"]
pub mod vote_spec;
verus! {
pub open spec fn positive_reply(m:io::Message)->bool {
    match m.body {io::MessageBody::VoteResponse{granted}=>granted,
        io::MessageBody::AppendResponse{accepted,..}=>accepted,_=>false}
}
pub open spec fn matched_durable_callback(n:state::Observation,event:io::Event)->bool {
    match event {io::Event::Complete{operation,kind,outcome}=>
        !n.recovery_required && kind==io::CompletionKind::Persist && outcome==io::CompletionOutcome::Success
        && operation.incarnation==n.incarnation && n.pending_write.is_some()
        && n.pending_write.unwrap().operation==operation,_=>false}
}
/// Protected local behavior target. Tests additionally require actual progress.
pub fn check_durable_reply(n:&mut state::Node,event:io::Event)->(result:Result<Vec<io::Effect>,io::InputError>)
    requires old(n).wf(),
    ensures n.wf(), result.is_ok() ==> forall|i:int| 0<=i<result.unwrap().len() ==>
        match #[trigger] result.unwrap()@[i] {io::Effect::Send{message,..}=>positive_reply(message) ==> matched_durable_callback(old(n).view(),event),_=>true},
{ n.handle_event(event) }
}
