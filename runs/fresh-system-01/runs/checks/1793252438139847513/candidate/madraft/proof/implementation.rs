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
pub enum Input {
    Timeout,
    Vote { term: u64, candidate: usize, last_term: u64, last_index: u64 },
    Reply { term: u64, peer: usize, granted: bool },
    Heartbeat { term: u64, leader: usize },
    Crash,
    Restart,
}
pub struct Action { pub who: targets::Key, pub input: Input }
pub struct Execution {
    pub states: Seq<Map<targets::Key, election::Image>>,
    pub actions: Seq<Action>,
}
pub open spec fn initialized(s: Map<targets::Key, election::Image>) -> bool {
    forall |k: targets::Key| s.contains_key(k) ==> {
        let v = s[k];
        v.term == 0 && v.role == election::Role::Follower && v.vote.is_none()
        && v.me == k.node && v.me < v.members && v.members > 0
        && v.last_index == 0 && v.last_term == 0 && v.count == 0
        && v.grants.len() == v.members
        && forall |i: int| 0 <= i < v.grants.len() ==> !v.grants[i]
    }
}
// Full transition equations and induction are developed here, rather than
// assuming reachable-state safety. This deliberately unfinished relation is
// broad: it retains all inputs and does not impose any target as a premise.
pub open spec fn permitted(before: election::Image, input: Input, after: election::Image) -> bool {
    after.me == before.me && after.members == before.members
    && after.last_index == before.last_index && after.last_term == before.last_term
    && after.term >= before.term
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
