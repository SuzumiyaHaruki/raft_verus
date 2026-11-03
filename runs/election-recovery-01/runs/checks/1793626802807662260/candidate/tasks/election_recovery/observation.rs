//! Fixed observation of state changes; no protocol decision or Raft invariant.
use super::{vote_api::*, targets};
use vstd::prelude::*;
use super::vote_api::View as StateView;
verus! {
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Change { pub node:u64, pub before:StateView, pub after:StateView }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Election { pub node:u64, pub term:u64 }
pub open spec fn elects(c:Change) -> bool {
    c.after.role==Role::Leader
        && (c.before.role!=Role::Leader || c.before.state.term!=c.after.state.term)
}
// Observe even an illegal Leader-to-Leader term change. Do not filter it out.
pub fn observe(c:Change) -> (out:Option<Election>)
    ensures out == if elects(c) { Some(Election{node:c.node,term:c.after.state.term}) } else { None },
{
    if matches!(c.after.role,Role::Leader)
        && (!matches!(c.before.role,Role::Leader) || c.before.state.term!=c.after.state.term) {
        Some(Election{node:c.node,term:c.after.state.term})
    } else { None }
}
pub open spec fn event(c:Change,group:targets::Group) -> targets::Event {
    if elects(c) { targets::Event::Elect{
        who:targets::Key{group,node:c.node as nat},term:c.after.state.term as nat} }
    else { targets::Event::Silent }
}
pub open spec fn projection(changes:Seq<Change>,group:targets::Group) -> Seq<targets::Event> {
    changes.map_values(|c:Change| event(c,group))
}
pub open spec fn historical_unique(changes:Seq<Change>) -> bool {
    forall|i:int,j:int| #![trigger changes[i],changes[j]]
        0<=i<changes.len() && 0<=j<changes.len() && elects(changes[i]) && elects(changes[j])
        && changes[i].after.state.term==changes[j].after.state.term ==> changes[i].node==changes[j].node
}
// Vocabulary bridge only. It supplies no election justification or Raft proof.
// No invented logs, states or snapshots: the target reads just these events.
pub proof fn bridge(changes:Seq<Change>,group:targets::Group,h:targets::History)
    requires historical_unique(changes), h.events==projection(changes,group),
    ensures targets::election_safety(h),
{
    assert forall|i:int,j:int| #![trigger h.events[i],h.events[j]]
        0<=i<h.events.len() && 0<=j<h.events.len() implies
        match (h.events[i],h.events[j]) {
            (targets::Event::Elect{who:a,term:ta},targets::Event::Elect{who:b,term:tb}) =>
                a.group==b.group && ta==tb ==> a.node==b.node,
            _ => true,
        } by {
        assert(h.events[i]==event(changes[i],group));
        assert(h.events[j]==event(changes[j],group));
    }
}
}
