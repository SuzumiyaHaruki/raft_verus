//! Definition regressions, NOT proofs of the student implementation.
#[path = "targets.rs"]
mod targets;
use targets::*;
use vstd::prelude::*;

verus! {
spec fn key(n: nat) -> Key { Key { group: 0, node: n } }
spec fn entry(byte: u8) -> Entry { Entry { term: 1, command: seq![byte] } }
spec fn view(term: nat, log: Seq<Entry>) -> targets::View {
    targets::View { term, leader: true, log, snapshot: None }
}
spec fn hist(events: Seq<Event>, states: Seq<Map<Key, targets::View>>) -> History {
    History { events, states }
}

proof fn two_leaders_in_one_term_are_rejected() {
    let h = hist(seq![Event::Elect { who: key(0), term: 1 }, Event::Silent,
        Event::Elect { who: key(1), term: 1 }], Seq::empty());
    assert(!election_safety(h));
}

proof fn different_entries()
    ensures entry(10) != entry(20), seq![10u8] != seq![20u8],
{
    assert(entry(10).command[0] == 10);
    assert(entry(20).command[0] == 20);
}

proof fn leader_rewrite_is_rejected() {
    different_entries();
    let before = map![key(0) => view(1, seq![entry(10)])];
    let after = map![key(0) => view(1, seq![entry(20)])];
    let h = hist(seq![Event::Silent], seq![before, after]);
    assert(h.states[0][key(0)].log[0] != h.states[1][key(0)].log[0]);
    assert(!leader_append_only(h));
}

proof fn equal_term_and_index_with_different_contents_are_rejected() {
    different_entries();
    let a = view(1, seq![entry(10)]);
    let b = view(1, seq![entry(20)]);
    let h = hist(Seq::empty(), seq![map![key(0) => a, key(1) => b]]);
    assert(at(h.states[0][key(0)].log, 1) == entry(10));
    assert(at(h.states[0][key(1)].log, 1) == entry(20));
    assert(h.states[0][key(0)].log.take(1)[0] != h.states[0][key(1)].log.take(1)[0]);
    assert(!log_matching(h));
}

proof fn committed_entry_missing_from_later_leader_is_rejected() {
    let first = map![key(0) => view(1, seq![entry(10)])];
    let later = map![key(1) => view(2, Seq::empty())];
    let h = hist(seq![Event::Commit { who: key(0), term: 1, through: 1 },
        Event::Elect { who: key(1), term: 2 }], seq![first, first, later]);
    assert(shape(h));
    assert(!leader_completeness(h));
}

proof fn conflicting_applications_across_restart_are_rejected() {
    different_entries();
    let h = hist(seq![Event::Apply { who: key(0), index: 1, entry: entry(10) },
        Event::Silent, Event::Apply { who: key(0), index: 1, entry: entry(20) }], Seq::empty());
    assert(applied(h.events[0], 0, 1) == Some(entry(10)));
    assert(applied(h.events[2], 0, 1) == Some(entry(20)));
    assert(!state_machine_safety(h));
}

proof fn unrelated_groups_do_not_conflict() {
    let h = hist(seq![Event::Elect { who: key(0), term: 1 },
        Event::Elect { who: Key { group: 1, node: 1 }, term: 1 }], Seq::empty());
    assert(election_safety(h));
}

proof fn snapshot_decodes_state_not_history() {
    different_entries();
    let app = Application { initial: Seq::empty(),
        step: |s: Seq<u8>, e: Entry| e.command,
        decode: |bytes: Seq<u8>| Some(bytes) };
    let entries = seq![entry(10), entry(20)];
    let good = Snapshot { through: 2, last_term: 1, bytes: seq![20u8] };
    let bad = Snapshot { through: 2, last_term: 1, bytes: seq![10u8] };
    reveal_with_fuel(fold, 3);
    assert(snapshot_represents(app, good, entries));
    assert(!snapshot_represents(app, bad, entries));
}
} // verus!
