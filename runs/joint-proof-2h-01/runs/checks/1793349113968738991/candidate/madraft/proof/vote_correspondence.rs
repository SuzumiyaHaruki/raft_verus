//! Parent of initialization: local empty-log RequestVote segment only.
#[path = "initialization.rs"]
pub mod initialization;
use initialization::{state, targets, observation, initialized_view};
use vstd::prelude::*;
verus! {
pub proof fn vote_observation(s: state::State, q: state::VoteRequest,
    t: state::State, r: state::VoteReply)
    requires state::well_formed(s), state::vote_step(s, q, t, r),
    ensures observation(t).term >= observation(s).term,
        r.granted ==> t.voted_for == Some(q.candidate) && t.term == q.term,
        s.voted_for.is_some() && t.term == s.term ==> t.voted_for == s.voted_for,
        t.me == s.me, t.members == s.members,
{}

pub fn initialized_request(me: usize, members: usize, q: state::VoteRequest)
    -> (result: Option<(state::State, state::VoteReply)>)
    ensures result.is_some() == (members > 0 && me < members),
        result.is_some() ==> state::well_formed(result.unwrap().0)
            && !observation(result.unwrap().0).leader
            && (result.unwrap().1.granted ==> result.unwrap().0.voted_for == Some(q.candidate)),
{
    match state::State::new(me, members) {
        None => None,
        Some(mut s) => {
            let ghost before = s;
            proof { initialized_view(s, me, members); }
            let r = s.request_vote(q);
            proof { vote_observation(before, q, s, r); }
            Some((s, r))
        }
    }
}
}
