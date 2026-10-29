//! Increment 1: election decision segments only. This is NOT yet an adapter for
//! every asynchronous Raft execution. See PROGRESS.md for the open coverage.
#[path = "../src/raft/election.rs"]
pub mod election;
#[path = "segments.rs"]
mod segments;
pub use segments::{Input, Output, Message, initial, step, receive, emit};
use crate::support::*;
use vstd::prelude::*;
verus! {
pub type State = election::Election;
pub type RuntimeState = election::Election;
pub open spec fn runtime_initial(s: RuntimeState) -> bool { initial(s) }
pub open spec fn runtime_step(s: RuntimeState,i: Input,t: RuntimeState,o: Output) -> bool { step(s,i,t,o) }
pub open spec fn view(s: RuntimeState) -> State { s }
// No global Raft invariant is claimed for this partial decision adapter.
pub open spec fn runtime_invariant(s: RuntimeState, sent: Seq<Envelope<Message>>) -> bool { s.term <= u64::MAX }
pub proof fn prove_runtime_initial(s: RuntimeState)
    requires runtime_initial(s),
    ensures runtime_invariant(s,Seq::empty()), initial(view(s)),
{}
pub proof fn prove_runtime_step(s: RuntimeState,i: Input,t: RuntimeState,o: Output,
    a: Seq<Envelope<Message>>,b: Seq<Envelope<Message>>)
    requires runtime_invariant(s,a), runtime_step(s,i,t,o), exchange(a,receive(i),emit(o),b),
    ensures runtime_invariant(t,b), step(view(s),i,view(t),o),
{}
}
