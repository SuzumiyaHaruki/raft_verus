//! Local executable initialization and observation; no claim about recovery.
#[path = "../src/raft/state.rs"]
pub mod state;
#[path = "../../requirements/targets.rs"]
pub mod targets;
use vstd::prelude::*;
verus! {
pub open spec fn observation(s: state::State) -> targets::View {
    targets::View { term: s.term as nat, leader: s.role == state::Role::Leader,
        log: Seq::empty(), snapshot: None }
}

pub proof fn initialized_view(s: state::State, me: usize, members: usize)
    requires state::initial(s, me, members),
    ensures state::well_formed(s), !observation(s).leader,
        observation(s).term == 0, observation(s).log.len() == 0,
        observation(s).snapshot.is_none(), s.voted_for.is_none(),
{}

pub fn construct_observed(me: usize, members: usize) -> (r: Option<state::State>)
    ensures r.is_some() == (members > 0 && me < members),
        r.is_some() ==> state::initial(r.unwrap(), me, members)
            && !observation(r.unwrap()).leader && observation(r.unwrap()).log.len() == 0,
{
    let r = state::State::new(me, members);
    proof {
        if r.is_some() { initialized_view(r.unwrap(), me, members); }
    }
    r
}
}
