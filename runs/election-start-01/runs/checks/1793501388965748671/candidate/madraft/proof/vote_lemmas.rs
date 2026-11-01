//! Consequences of the invariant maintained by the actual executable Core.
use crate::{vote_api::*, vote_core::Core};
use vstd::prelude::*;
verus! {
pub proof fn uniqueness(tracked c: &Core) ensures unique(c.choices()) { c.facts(); }
pub proof fn restart_composition(tracked before: &Core, tracked after: &Core, record: Record)
    requires before.view().state == record.state, before.view().next == record.op + 1,
        after.base() == Some(record), before.view().config == after.view().config,
    ensures unique(before.choices() + after.choices()),
{
    before.facts(); after.facts();
    let a = before.choices(); let b = after.choices();
    assert forall|i:int,j:int| 0 <= i < (a+b).len() && 0 <= j < (a+b).len()
        && (#[trigger] (a+b)[i]).node == (#[trigger] (a+b)[j]).node && (a+b)[i].term == (a+b)[j].term
        implies (a+b)[i].candidate == (a+b)[j].candidate by {
        if i < a.len() && j < a.len() { assert(a[i].candidate == a[j].candidate); }
        else if i >= a.len() && j >= a.len() { assert(b[i-a.len()].candidate == b[j-a.len()].candidate); }
        else if i < a.len() {
            assert(a[i].term <= record.state.term);
            assert(b[j-a.len()].term >= record.state.term);
            assert(record.state.voted_for == Some(a[i].candidate));
        } else {
            assert(a[j].term <= record.state.term);
            assert(b[i-a.len()].term >= record.state.term);
            assert(record.state.voted_for == Some(a[j].candidate));
        }
    }
}

// Audit every public mutation using actual calls. This is independent of which
// operation the async adapter selects and requires no protocol-state premise.
pub fn audit_step(c: &mut Core, event: Event)
    ensures c.view().config == old(c).view().config,
        c.view().state.log == old(c).view().state.log,
        c.view().state.term >= old(c).view().state.term,
        c.view().next >= old(c).view().next,
        c.view().state.term == old(c).view().state.term
            && old(c).view().state.voted_for.is_some()
            ==> c.view().state.voted_for == old(c).view().state.voted_for,
        old(c).view().halted ==> c.view() == old(c).view(),
        c.base() == old(c).base(), unique(c.choices()),
{
    proof { c.facts(); }
    let ghost before = c.view();
    match event {
        Event::Start => {
            let out = c.start_election();
            if let Campaign::Save(w) = out {
                assert(w.record.op == before.next);
                assert(c.view().next == before.next + 1);
                assert(c.view().state.term == before.state.term + 1);
            }
        },
        Event::Receive(r) => {
            let out = c.receive(r);
            if let Receive::Save(w) = out {
                assert(w.record.op == before.next);
                assert(c.view().next == before.next + 1);
            }
        },
        Event::Complete(n) => {
            let out = c.complete(n);
            assert(c.view().state == before.state && c.view().next == before.next);
            assert(out.is_some() ==> n.success && before.pending.is_some()
                && before.pending.unwrap().record.op == n.op);
        },
    }
    proof { c.facts(); }
}

// The invariant is available after any actual sequence, not just immediately
// after recover. A stale notice cannot clear a newer Pending or emit an action.
pub fn stale_after_recovery(c: &mut Core, notice: Completion) -> (out: Option<Action>)
    requires old(c).base().is_some(), notice.op <= old(c).base().unwrap().op,
    ensures out.is_none(), c.view() == old(c).view(),
        c.choices() == old(c).choices(), c.base() == old(c).base(),
{
    proof { c.facts(); }
    c.complete(notice)
}

}
