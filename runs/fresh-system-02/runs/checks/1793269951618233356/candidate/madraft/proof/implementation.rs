//! Increment 1: election decision segments only. This is NOT yet an adapter for
//! every asynchronous Raft execution. See PROGRESS.md for the open coverage.
#[path = "../src/raft/election.rs"]
pub mod election;
#[path = "segments.rs"]
mod segments;
#[path = "../src/raft/log.rs"]
pub mod log_core;
pub use segments::{State, Input, Output, Message, initial, step, receive, emit};
use crate::support::*;
use vstd::prelude::*;
verus! {
pub type RuntimeState = log_core::Core;
pub open spec fn runtime_initial(s: RuntimeState) -> bool { initial(view(s)) }
pub open spec fn runtime_step(s: RuntimeState,i: Input,t: RuntimeState,o: Output) -> bool { step(view(s),i,view(t),o) }
pub open spec fn view(s: RuntimeState) -> State { State {election:s.election, log:s.log@, commit:s.commit,applied:s.applied} }
// No global Raft invariant is claimed for this partial decision adapter.
pub open spec fn runtime_invariant(s: RuntimeState, sent: Seq<Envelope<Message>>) -> bool { s.election.term <= u64::MAX }
pub proof fn prove_runtime_initial(s: RuntimeState)
    requires runtime_initial(s),
    ensures runtime_invariant(s,Seq::empty()), initial(view(s)),
{}
pub proof fn prove_runtime_step(s: RuntimeState,i: Input,t: RuntimeState,o: Output,
    a: Seq<Envelope<Message>>,b: Seq<Envelope<Message>>)
    requires runtime_invariant(s,a), runtime_step(s,i,t,o), exchange(a,receive(i),emit(o),b),
    ensures runtime_invariant(t,b), step(view(s),i,view(t),o),
{}
pub fn actual_initial() -> (c: RuntimeState)
    ensures runtime_initial(c),
{ log_core::new_core() }
pub fn actual_proposal(c: &mut RuntimeState,entry: log_core::Entry) -> (ok: bool)
    requires old(c).election.role == election::Role::Leader,
    ensures ok ==> runtime_step(*old(c),Input::Propose {entry},*c,Seq::empty()),
        !ok ==> c.log@ == old(c).log@,
{
    log_core::propose(&mut c.log,entry)
}
pub fn actual_insert(c: &mut RuntimeState,at: usize,entry: log_core::Entry) -> (ok: bool)
    requires !(old(c).election.role == election::Role::Leader),
    ensures runtime_step(*old(c),Input::Insert {at:at as nat,entry},*c,Seq::empty()),
{
    log_core::insert(&mut c.log,at,entry)
}

}
