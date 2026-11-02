
#[path = "/home/nitro/Desktop/raft_verus/requirements/targets.rs"] mod targets;
use targets::*;
use vstd::prelude::*;
verus! {
proof fn incorrect_claim() {
    let h = History { states: Seq::empty(), events: seq![
        Event::Elect { who: Key { group: 1, node: 0 }, term: 2 },
        Event::Elect { who: Key { group: 1, node: 1 }, term: 2 }] };
    assert(election_safety(h));
}
}
