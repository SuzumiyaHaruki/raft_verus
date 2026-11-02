//! DRAFT independent behavioral targets. This is an observation vocabulary,
//! not a prescribed executable representation or a second protocol engine.
use vstd::prelude::*;

verus! {
pub type Group = nat;
pub type Node = nat;
pub type Term = nat;
pub type Index = nat;

pub struct Key { pub group: Group, pub node: Node }
pub struct Entry { pub term: Term, pub command: Seq<u8> }
pub struct Snapshot {
    pub through: Index,
    pub last_term: Term,
    pub bytes: Seq<u8>,
}
pub struct View {
    pub term: Term,
    pub leader: bool,
    // The logical prefix includes compacted entries. It is NOT a free witness:
    // the generated refinement must connect it to actual execution and storage.
    pub log: Seq<Entry>,
    pub snapshot: Option<Snapshot>,
}
pub enum Event {
    Silent,
    Elect { who: Key, term: Term },
    // Observe actual commit-index advancement, never future preservation.
    Commit { who: Key, term: Term, through: Index },
    Apply { who: Key, index: Index, entry: Entry },
    // Logical application of the prefix represented by an installed snapshot.
    ApplySnapshot { who: Key, prefix: Seq<Entry>, snapshot: Snapshot },
    CreateSnapshot { who: Key, prefix: Seq<Entry>, snapshot: Snapshot },
    PersistSnapshot { who: Key, prefix: Seq<Entry>, snapshot: Snapshot },
    RestoreSnapshot { who: Key, prefix: Seq<Entry>, snapshot: Snapshot },
}
pub struct History {
    // before/after each linearized action, including crash and restart.
    pub states: Seq<Map<Key, View>>,
    pub events: Seq<Event>,
}

pub open spec fn prefix(a: Seq<Entry>, b: Seq<Entry>) -> bool {
    a.len() <= b.len() && a == b.take(a.len() as int)
}
pub open spec fn at(log: Seq<Entry>, index: Index) -> Entry
    recommends 1 <= index <= log.len(),
{ log[index as int - 1] }

pub open spec fn shape(h: History) -> bool {
    h.states.len() == h.events.len() + 1
    && forall |t: int| 0 <= t < h.events.len() ==> match #[trigger] h.events[t] {
        Event::Elect { who, term } =>
            h.states[t + 1].contains_key(who)
            && h.states[t + 1][who].leader
            && h.states[t + 1][who].term == term,
        Event::Commit { who, term, through } =>
            h.states[t + 1].contains_key(who)
            && h.states[t + 1][who].term == term
            && 1 <= through <= h.states[t + 1][who].log.len(),
        Event::Apply { who: _, index, entry: _ } => index >= 1,
        _ => true,
    }
}

pub open spec fn election_safety(h: History) -> bool {
    forall |i: int, j: int| #![trigger h.events[i], h.events[j]] 0 <= i < h.events.len() && 0 <= j < h.events.len() ==>
        match (h.events[i], h.events[j]) {
            (Event::Elect { who: a, term: ta }, Event::Elect { who: b, term: tb }) =>
                a.group == b.group && ta == tb ==> a.node == b.node,
            _ => true,
        }
}

pub open spec fn leader_append_only(h: History) -> bool {
    forall |t: int, k: Key| #![trigger h.states[t][k]] 0 <= t < h.events.len()
        && h.states[t].contains_key(k) && h.states[t + 1].contains_key(k)
        && h.states[t][k].leader && h.states[t + 1][k].leader
        && h.states[t][k].term == h.states[t + 1][k].term
        ==> prefix(h.states[t][k].log, h.states[t + 1][k].log)
}

pub open spec fn log_matching(h: History) -> bool {
    // Compare represented logs across time as well as between nodes.
    forall |s: int, t: int, a: Key, b: Key, index: Index|
        #![trigger at(h.states[s][a].log, index), at(h.states[t][b].log, index)]
        0 <= s < h.states.len() && 0 <= t < h.states.len()
        && a.group == b.group && h.states[s].contains_key(a) && h.states[t].contains_key(b)
        && 1 <= index <= h.states[s][a].log.len() && index <= h.states[t][b].log.len()
        && at(h.states[s][a].log, index).term == at(h.states[t][b].log, index).term
        ==> h.states[s][a].log.take(index as int) == h.states[t][b].log.take(index as int)
}

pub open spec fn leader_completeness(h: History) -> bool {
    forall |c: int, e: int| #![trigger h.events[c], h.events[e]] 0 <= c < h.events.len() && 0 <= e < h.events.len() ==>
        match (h.events[c], h.events[e]) {
            (Event::Commit { who: a, term: tc, through }, Event::Elect { who: b, term: te }) =>
                a.group == b.group && tc < te ==>
                    h.states[e + 1].contains_key(b)
                    && through <= h.states[e + 1][b].log.len()
                    && h.states[c + 1][a].log.take(through as int)
                        == h.states[e + 1][b].log.take(through as int),
            _ => true,
        }
}

pub open spec fn applied(event: Event, group: Group, index: Index) -> Option<Entry> {
    match event {
        Event::Apply { who, index: i, entry } =>
            if who.group == group && i == index { Some(entry) } else { None },
        Event::ApplySnapshot { who, prefix, snapshot: _ } =>
            if who.group == group && 1 <= index <= prefix.len() {
                Some(at(prefix, index))
            } else { None },
        _ => None,
    }
}

pub open spec fn state_machine_safety(h: History) -> bool {
    forall |a: int, b: int, group: Group, index: Index|
        0 <= a < h.events.len() && 0 <= b < h.events.len()
        && applied(h.events[a], group, index).is_some()
        && applied(h.events[b], group, index).is_some()
        ==> applied(h.events[a], group, index) == applied(h.events[b], group, index)
}

// The application semantics is selected per group and refined to the actual
// service. Decoding recovers application state, not historical commands.
pub struct Application {
    pub initial: Seq<u8>,
    pub step: spec_fn(Seq<u8>, Entry) -> Seq<u8>,
    pub decode: spec_fn(Seq<u8>) -> Option<Seq<u8>>,
}
pub open spec fn fold(app: Application, entries: Seq<Entry>) -> Seq<u8>
    decreases entries.len(),
{
    if entries.len() == 0 { app.initial }
    else { (app.step)(fold(app, entries.drop_last()), entries.last()) }
}
pub open spec fn snapshot_represents(app: Application, snap: Snapshot, entries: Seq<Entry>) -> bool {
    snap.through == entries.len()
    && snap.last_term == (if entries.len() == 0 { 0nat } else { entries.last().term })
    && (app.decode)(snap.bytes) == Some(fold(app, entries))
}
pub open spec fn committed_prefix_before(h: History, time: int, group: Group, entries: Seq<Entry>) -> bool {
    entries.len() == 0 || exists |c: int| #![trigger h.events[c]] 0 <= c < time && match h.events[c] {
        Event::Commit { who, term: _, through } => who.group == group
            && entries.len() <= through && h.states[c + 1].contains_key(who)
            && prefix(entries, h.states[c + 1][who].log),
        _ => false,
    }
}
pub open spec fn snapshot_fidelity(h: History, apps: Map<Group, Application>) -> bool {
    (forall |t: int, k: Key| #![trigger h.states[t][k]] 0 <= t < h.states.len() && h.states[t].contains_key(k) ==>
        match h.states[t][k].snapshot {
            None => true,
            Some(s) => apps.contains_key(k.group) && s.through <= h.states[t][k].log.len()
                && snapshot_represents(apps[k.group], s, h.states[t][k].log.take(s.through as int))
                && committed_prefix_before(h, t, k.group, h.states[t][k].log.take(s.through as int)),
        })
    && (forall |t: int| 0 <= t < h.events.len() ==> match #[trigger] h.events[t] {
        Event::ApplySnapshot { who, prefix, snapshot }
        | Event::CreateSnapshot { who, prefix, snapshot }
        | Event::PersistSnapshot { who, prefix, snapshot }
        | Event::RestoreSnapshot { who, prefix, snapshot } =>
            apps.contains_key(who.group) && snapshot_represents(apps[who.group], snapshot, prefix)
                && committed_prefix_before(h, t, who.group, prefix),
        _ => true,
    })
}
} // verus!
