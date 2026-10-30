//! Actual log mutations, shared by Rust execution and Verus.
use vstd::prelude::*;
verus! {
#[cfg_attr(not(verus_keep_ghost), derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize))]
pub(super) struct LogEntry { pub(super) term: u64, pub(super) data: Vec<u8> }

pub(super) fn fresh_log() -> (log: Vec<LogEntry>)
    ensures log.len() == 1, log@[0].term == 0, log@[0].data@ == Seq::<u8>::empty(),
{
    let mut log = Vec::new();
    log.push(LogEntry { term: 0, data: Vec::new() });
    log
}

pub(super) fn same(a: &LogEntry, b: &LogEntry) -> (yes: bool)
    ensures yes == (a.term == b.term && a.data@ == b.data@),
{
    if a.term != b.term || a.data.len() != b.data.len() { return false; }
    let mut i = 0;
    while i < a.data.len()
        invariant i <= a.data.len(), a.data.len() == b.data.len(),
            forall|j: int| 0 <= j < i ==> a.data@[j] == b.data@[j],
        decreases a.data.len() - i,
    {
        if a.data[i] != b.data[i] { return false; }
        i += 1;
    }
    proof { assert(a.data@ =~= b.data@); }
    true
}

// No unchecked helper preconditions: malformed or exhausted boundaries reject.
// Real role authorization is the caller's State::is_leader result; its
// constructor/recovery composition is still a separate proof obligation.
pub(super) fn propose(log: &mut Vec<LogEntry>, base: usize, term: u64, leader: bool, data: Vec<u8>) -> (accepted: bool)
    ensures accepted ==> leader && log.len() == old(log).len() + 1
        && log@.take(old(log).len() as int) == old(log)@
        && log@[old(log).len() as int].term == term
        && log@[old(log).len() as int].data@ == data@,
        !accepted ==> log@ == old(log)@,
{
    if !leader || log.len() == 0 || log.len() >= usize::MAX - base { return false; }
    log.push(LogEntry { term, data });
    true
}

pub(super) fn truncate_conflict(log: &mut Vec<LogEntry>, base: usize, committed: usize, offset: usize, incoming: &LogEntry) -> (changed: bool)
    ensures !changed ==> log@ == old(log)@,
        changed ==> offset < old(log).len() && log@ == old(log)@.take(offset as int)
            && base + offset > committed && old(log)@[offset as int].term != incoming.term,
        // Exact retained content, including each command's bytes.
        forall|i: int| 0 <= i < log.len() ==> log@[i] == old(log)@[i],
{
    if offset >= log.len() || base > usize::MAX - offset { return false; }
    if base + offset <= committed || log[offset].term == incoming.term { return false; }
    log.truncate(offset);
    true
}

pub(super) fn append_received(log: &mut Vec<LogEntry>, entry: LogEntry) -> (accepted: bool)
    ensures accepted == (old(log).len() < usize::MAX),
        accepted ==> log@ == old(log)@.push(entry),
        !accepted ==> log@ == old(log)@,
{
    if log.len() == usize::MAX { return false; }
    log.push(entry);
    true
}
// Actual proposal caller: derives helper inputs from real consensus state.
pub(super) fn propose_from_state(state: &super::state::State, log: &mut Vec<LogEntry>, base: usize, data: Vec<u8>) -> (index: Option<usize>)
    ensures index.is_some() ==> state.role is Leader
        && index.unwrap() == base + old(log).len()
        && log.len() == old(log).len() + 1
        && log@.take(old(log).len() as int) == old(log)@
        && log@[old(log).len() as int].term == state.term
        && log@[old(log).len() as int].data@ == data@,
        index.is_none() ==> log@ == old(log)@,
{
    if log.len() == 0 || log.len() >= usize::MAX - base { return None; }
    let index = base + log.len();
    let leader = state.is_leader();
    if propose(log, base, state.term, leader, data) { Some(index) } else { None }
}

// Actual received-entry call composition. The async handler separately checks
// term, predecessor and genuine-message provenance; those remain OPEN.
pub(super) fn receive_entry(log: &mut Vec<LogEntry>, base: usize, committed: usize, index: usize, entry: LogEntry) -> (accepted: bool)
    ensures !accepted ==> log@ == old(log)@,
        forall|i: int| 0 <= i < old(log).len() && base as int + i <= committed ==>
            i < log.len() && log@[i].term == old(log)@[i].term && log@[i].data@ == old(log)@[i].data@,
        accepted ==> index >= base && index - base < log.len()
        && log@[index as int-base as int].term == entry.term
        && log@[index as int-base as int].data@ == entry.data@,
{
    if index < base { return false; }
    let offset = index - base;
    if offset > log.len() { return false; }
    if offset < log.len() {
        if same(&log[offset], &entry) { return true; }
        if !truncate_conflict(log, base, committed, offset, &entry) { return false; }
    }
    append_received(log, entry)
}

// Local snapshot creation uses the actual applied and committed cursors.
pub(super) fn compact_committed(log: &mut Vec<LogEntry>, base: usize, index: usize, applied: usize, committed: usize) -> (changed: bool)
    ensures !changed ==> log@ == old(log)@,
        changed ==> base <= index && index <= applied && index <= committed
            && index - base < old(log).len()
            && log.len() == old(log).len() - (index-base)
            && log.len() > 0 && log@[0].term == old(log)@[index as int-base as int].term
            && log@[0].data@ == Seq::<u8>::empty(),
        changed ==> forall|j: int| 1 <= j < log.len() ==>
            log@[j] == old(log)@[j + index as int-base as int],
{
    if log.len() == 0 || log.len() > usize::MAX - base { return false; }
    let last = base + log.len() - 1;
    if index < base || index > last || index > applied || index > committed { return false; }
    let offset = index-base;
    let mut tail = log.split_off(offset);
    let mut boundary = tail.remove(0);
    boundary.data.clear();
    tail.insert(0, boundary);
    *log = tail;
    true
}

pub(super) open spec fn valid_image(log: Seq<LogEntry>, base: usize, committed: usize, term: u64) -> bool {
    log.len() > 0 && log.len() <= usize::MAX-base
    && log[0].data@ == Seq::<u8>::empty() && (base == 0 ==> log[0].term == 0)
    && base <= committed < base + log.len()
    && (forall|i: int| 0 <= i < log.len() ==> log[i].term <= term)
    && (forall|i: int| 1 <= i < log.len() ==> log[i-1].term <= #[trigger] log[i].term)
}
// Actual post-deserialization validator. No desired consensus invariant is
// assumed: history provenance and committed-prefix origin remain separate.
pub(super) fn validate_image(log: &Vec<LogEntry>, base: usize, committed: usize, term: u64) -> (valid: bool)
    ensures valid ==> valid_image(log@, base, committed, term),
{
    if log.len() == 0 || log.len() > usize::MAX-base { return false; }
    if log[0].data.len() != 0 || base == 0 && log[0].term != 0 { return false; }
    if committed < base || committed >= base + log.len() { return false; }
    let mut i = 0usize;
    while i < log.len()
        invariant i <= log.len(),
            forall|j: int| 0 <= j < i ==> log@[j].term <= term,
            forall|j: int| 1 <= j < i ==> log@[j-1].term <= #[trigger] log@[j].term,
        decreases log.len()-i,
    {
        if log[i].term > term { return false; }
        if i > 0 && log[i-1].term > log[i].term { return false; }
        i += 1;
    }
    true
}

// Actual recovery caller composition, supplied values decoded by the host.
pub(super) fn recover_decoded(previous: &super::state::State, term: u64, vote: Option<u64>, members: u64, log: &Vec<LogEntry>, base: usize, committed: usize) -> (state: Option<super::state::State>)
    ensures state.is_some() ==> valid_image(log@, base, committed, term)
        && state.unwrap().role is Follower && state.unwrap().term == term
        && state.unwrap().voted_for == vote && term >= previous.term,
        state.is_some() && term == previous.term && previous.voted_for.is_some()
            ==> vote == previous.voted_for,
{
    let restored = match previous.restore_successor(term, vote, members) {
        Some(state) => state,
        None => return None,
    };
    if !validate_image(log, base, committed, term) { return None; }
    Some(restored)
}

}
