//! Incremental execution interface. This unit does not yet claim full refinement.
//! All safety theorems remain open until message/storage/host induction is closed.
#[path = "../src/raft/election.rs"]
pub mod election;
use vstd::prelude::*;
use crate::targets;

verus! {
// An overapproximation of the currently implemented election boundary. Network
// authenticity and durable write completion will constrain these actions in
// the next increment. In particular, Reply is not yet a quorum certificate.
pub use election::Input;
pub struct Action { pub who: targets::Key, pub input: Input }
pub struct Execution {
    pub states: Seq<Map<targets::Key, election::Image>>,
    pub actions: Seq<Action>,
}
pub open spec fn initialized(s: Map<targets::Key, election::Image>) -> bool {
    forall |k: targets::Key| s.contains_key(k) ==> {
        let v = s[k];
        election::initial(v) && v.me == k.node
    }
}
// Full transition equations and induction are developed here, rather than
// assuming reachable-state safety. This deliberately unfinished relation is
// broad: it retains all inputs and does not impose any target as a premise.
pub open spec fn permitted(before: election::Image, input: Input, after: election::Image) -> bool {
    match input {
        // Broad recovery abstraction: a crash may lose a volatile term or vote.
        // This does NOT assert that an arbitrary recovered vote is durable.
        // Storage provenance is still open, so no safety theorem consumes it.
        Input::Crash | Input::Restart => election::well_formed(after)
            && after.me == before.me && after.members == before.members
            && after.last_index == 0 && after.last_term == 0
            && after.role == election::Role::Follower,
        _ => after == election::next(before, input),
    }
}
pub open spec fn execution(run: Execution) -> bool {
    run.states.len() == run.actions.len() + 1 && initialized(run.states[0])
    && forall |t: int| #![trigger run.actions[t]] 0 <= t < run.actions.len() ==> {
        let k = run.actions[t].who;
        run.states[t].contains_key(k) && run.states[t+1].contains_key(k)
        && run.states[t+1].dom() == run.states[t].dom()
        && permitted(run.states[t][k], run.actions[t].input, run.states[t+1][k])
        && forall |other: targets::Key| other != k && run.states[t].contains_key(other)
            ==> run.states[t+1][other] == run.states[t][other]
    }
}
pub proof fn state_induction(run: Execution, t: int)
    requires execution(run), 0 <= t < run.states.len(),
    ensures forall |k: targets::Key| run.states[t].contains_key(k)
        ==> election::well_formed(run.states[t][k]),
    decreases t,
{
    if t == 0 {
        assert forall |k: targets::Key| run.states[t].contains_key(k)
            implies election::well_formed(run.states[t][k]) by {
            assert(election::initial(run.states[0][k]));
        }
    } else {
        state_induction(run, t-1);
        let action = run.actions[t-1];
        assert forall |k: targets::Key| run.states[t].contains_key(k)
            implies election::well_formed(run.states[t][k]) by {
            assert(run.states[t-1].contains_key(k));
            if k == action.who {
                let before = run.states[t-1][k];
                assert(permitted(before, action.input, run.states[t][k]));
                election::next_preserves(before, action.input);
            } else { assert(run.states[t][k] == run.states[t-1][k]); }
        }
    }
}
pub open spec fn view(v: election::Image) -> targets::View {
    targets::View { term: v.term as nat, leader: v.role == election::Role::Leader,
        log: Seq::empty(), snapshot: None }
}
pub open spec fn observe(run: Execution) -> targets::History {
    targets::History {
        states: Seq::new(run.states.len(), |t: int| Map::new(
            |k: targets::Key| run.states[t].contains_key(k), |k: targets::Key| view(run.states[t][k]))),
        events: Seq::new(run.actions.len(), |t: int| {
            let k = run.actions[t].who;
            if run.states[t].contains_key(k) && run.states[t+1].contains_key(k)
                && run.states[t][k].role != election::Role::Leader
                && run.states[t+1][k].role == election::Role::Leader {
                targets::Event::Elect { who: k, term: run.states[t+1][k].term as nat }
            } else { targets::Event::Silent }
        }),
    }
}
pub proof fn observation_shape(run: Execution)
    requires execution(run),
    ensures targets::shape(observe(run)),
{
    let h = observe(run);
    assert forall |t: int| 0 <= t < h.events.len() implies match #[trigger] h.events[t] {
        targets::Event::Elect { who, term } => h.states[t+1].contains_key(who)
            && h.states[t+1][who].leader && h.states[t+1][who].term == term,
        targets::Event::Commit { who, term, through } => h.states[t+1].contains_key(who)
            && h.states[t+1][who].term == term && 1 <= through <= h.states[t+1][who].log.len(),
        targets::Event::Apply { who: _, index, entry: _ } => index >= 1,
        _ => true,
    } by {
        let k = run.actions[t].who;
        assert(run.states[t+1].contains_key(k));
    }
}
}
