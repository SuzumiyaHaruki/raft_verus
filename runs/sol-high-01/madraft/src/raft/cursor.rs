//! Application and snapshot boundary transformations, used by the core.
use vstd::prelude::*;
use super::{log::{self, Entry}, state};
#[cfg(verus_keep_ghost)]
use super::log::{entry_view, log_view};
verus! {
pub struct Application { pub index: u64, pub entry: Entry }

pub fn next_application(log: &Vec<Entry>, base: u64, commit: u64, applied: &mut u64) -> (r: Option<Application>)
    requires base <= *old(applied) <= commit <= base as int + log.len() <= u64::MAX,
    ensures
        base <= *applied <= commit,
        r.is_some() == (*old(applied) < commit),
        r.is_none() ==> *applied == *old(applied),
        r.is_some() ==> *applied == *old(applied) + 1 && r.unwrap().index == *applied
            && entry_view(r.unwrap().entry) == entry_view(log@[(*applied - base - 1) as int]),
{
    if *applied >= commit { return None; }
    let index = *applied + 1;
    let entry = log::copy_entry(&log[(index - base - 1) as usize]);
    *applied = index;
    Some(Application { index, entry })
}

pub fn compact(log: &mut Vec<Entry>, base: &mut u64, base_term: &mut u64,
    applied: u64, commit: u64, index: u64) -> (ok: bool)
    requires *old(base) <= applied <= commit <= *old(base) as int + old(log).len() <= u64::MAX,
    ensures
        ok == (*old(base) < index <= applied),
        *base <= applied <= commit <= *base as int + log.len() <= u64::MAX,
        *base as int + log.len() == *old(base) as int + old(log).len(),
        ok ==> *base == index && *base_term == old(log)@[(index - *old(base) - 1) as int].term
            && log_view(log@) == log_view(old(log)@).skip((index - *old(base)) as int),
        !ok ==> log@ == old(log)@ && *base == *old(base) && *base_term == *old(base_term),
        forall|limit: u64| #[trigger] log::bounded(old(log)@, limit) && *old(base_term) <= limit
            ==> log::bounded(log@, limit) && *base_term <= limit,
{
    let end = *base + log.len() as u64;
    if !state::snapshot_authorized(*base, index, applied, commit, end) { return false; }
    let count = (index - *base) as usize;
    let term = log[count - 1].term;
    log::compact(log, count);
    *base = index;
    *base_term = term;
    true
}

// Snapshot semantic fidelity is a separate network/service invariant. This
// function proves the concrete retained suffix and cursor transformation only.
pub fn install(log: &mut Vec<Entry>, base: &mut u64, base_term: &mut u64,
    commit: &mut u64, applied: &mut u64, index: u64, term: u64) -> (changed: bool)
    requires *old(base) <= *old(applied) <= *old(commit) <= *old(base) as int + old(log).len() <= u64::MAX,
    ensures
        changed == (index > *old(commit)),
        *base <= *applied <= *commit <= *base as int + log.len() <= u64::MAX,
        *commit >= *old(commit), *applied >= *old(applied),
        changed ==> *base == index && *base_term == term && *commit == index && *applied == index,
        !changed ==> log@ == old(log)@ && *base == *old(base) && *base_term == *old(base_term)
            && *commit == *old(commit) && *applied == *old(applied),
        changed && *old(base) < index <= *old(base) as int + old(log).len()
            && old(log)@[(index - *old(base) - 1) as int].term == term
            ==> log_view(log@) == log_view(old(log)@).skip((index - *old(base)) as int),
        forall|limit: u64| #[trigger] log::bounded(old(log)@, limit) && *old(base_term) <= limit && term <= limit
            ==> log::bounded(log@, limit) && *base_term <= limit,
        changed && (index > *old(base) as int + old(log).len()
            || (*old(base) < index <= *old(base) as int + old(log).len()
                && old(log)@[(index - *old(base) - 1) as int].term != term)) ==> log.len() == 0,
{
    if index <= *commit { return false; }
    if log::term_at(log, *base, *base_term, index) == Some(term) {
        log::compact(log, (index - *base) as usize);
    } else { *log = Vec::new(); }
    *base = index;
    *base_term = term;
    *commit = index;
    *applied = index;
    true
}
}
