//! Parent use of the real begin/complete transition contracts.
//! Local operation histories; cluster transport and durable history remain OPEN.
#[path = "vote_operation.rs"]
pub mod vote_operation;
use vote_operation::{protocol as p, state};
use vstd::prelude::*;
verus! {
pub enum Input { Request(state::VoteRequest), Completion(u64, p::VoteImage) }
pub enum Output { Submission(Option<(u64, p::VoteImage)>), Reply(Option<state::VoteReply>) }
pub open spec fn step(s: p::OperationView, i: Input, t: p::OperationView, o: Output) -> bool {
    match (i, o) {
        (Input::Request(q), Output::Submission(op)) => p::begin_step(s, q, t, op),
        (Input::Completion(id, image), Output::Reply(r)) => p::complete_step(s, id, image, t, r),
        _ => false,
    }
}

pub proof fn initialize(v: p::OperationView, me: usize, members: usize)
    requires p::initial_view(v, me, members),
    ensures p::view_invariant(v),
{
    vote_operation::initialization::initialized_view(v.state, me, members);
}

pub proof fn preserve(s: p::OperationView, i: Input, t: p::OperationView, o: Output)
    requires p::view_invariant(s), step(s, i, t, o),
    ensures p::view_invariant(t),
{
}

// Every released reply has a retained request, exact completed image and
// checked voting transition. This deliberately does not infer that I/O occurred.
pub proof fn released_reply(s: p::OperationView, id: u64, image: p::VoteImage,
    t: p::OperationView, reply: state::VoteReply)
    requires p::view_invariant(s), p::complete_step(s, id, image, t, Some(reply)),
    ensures s.pending.is_some(), image == p::image(t.state),
        state::vote_step(s.pending.unwrap().4, s.pending.unwrap().2, t.state, reply),
        reply.granted ==> t.state.voted_for == Some(s.pending.unwrap().2.candidate),
{}
}
