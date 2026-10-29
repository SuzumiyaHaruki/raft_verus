//! Initial-state observation only. There is deliberately no execution predicate
//! here: host assembly, transition refinement and induction remain open.
use vstd::prelude::*;
use crate::{core::Core, state::Role, log::Entry, targets};
verus! {
pub open spec fn fresh(c: Core) -> bool {
    c.state.term == 0 && c.state.role is Follower && c.state.voted_for.is_none()
    && c.base == 0 && c.base_term == 0 && c.log.len() == 0 && c.snapshot.len() == 0
    && c.commit == 0 && c.applied == 0
}

pub open spec fn observe_fresh(c: Core) -> targets::View
    recommends fresh(c),
{
    targets::View {
        term: c.state.term as nat,
        leader: c.state.role is Leader,
        log: c.log@.map(|i: int, e: Entry| targets::Entry { term: e.term as nat, command: e.data@ }),
        snapshot: None,
    }
}

// Executable linkage for the base case: this calls the real constructor.
pub fn initialize(me: usize, members: usize) -> (c: Core)
    requires members > 0, me < members,
    ensures c.wf(), c.me == me, c.members == members, fresh(c),
        observe_fresh(c).term == 0, !observe_fresh(c).leader,
        observe_fresh(c).log.len() == 0, observe_fresh(c).snapshot.is_none(),
{ Core::new(me, members) }

pub open spec fn initial_history(nodes: Map<targets::Key, Core>) -> targets::History
    recommends forall|k: targets::Key| nodes.contains_key(k) ==> fresh(nodes[k]),
{
    targets::History {
        states: seq![Map::new(|k: targets::Key| nodes.contains_key(k), |k: targets::Key| observe_fresh(nodes[k]))],
        events: Seq::empty(),
    }
}

// Conditional base case only. The real host's assembly of this finite map is
// still an obligation; no nonempty trace or preservation theorem is asserted.
pub proof fn initial_observation(nodes: Map<targets::Key, Core>)
    requires nodes.dom().finite(), forall|k: targets::Key| nodes.contains_key(k) ==> fresh(nodes[k]),
    ensures targets::shape(initial_history(nodes)),
        targets::election_safety(initial_history(nodes)),
        targets::leader_append_only(initial_history(nodes)),
        targets::log_matching(initial_history(nodes)),
        targets::leader_completeness(initial_history(nodes)),
        targets::state_machine_safety(initial_history(nodes)),
        targets::snapshot_fidelity(initial_history(nodes), Map::empty()),
{
    let h = initial_history(nodes);
    assert forall|k: targets::Key| h.states[0].contains_key(k) implies h.states[0][k].log.len() == 0 by {
        assert(fresh(nodes[k]));
    }
}
}
