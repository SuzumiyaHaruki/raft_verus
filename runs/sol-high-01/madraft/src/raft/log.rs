//! Verified physical log operations; commands retain exact byte sequences.
use vstd::prelude::*;

verus! {
#[cfg_attr(not(verus_keep_ghost), derive(Clone, serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Eq)]
pub struct Entry { pub term: u64, pub data: Vec<u8> }

pub open spec fn entry_view(e: Entry) -> (u64, Seq<u8>) { (e.term, e.data@) }
pub open spec fn log_view(log: Seq<Entry>) -> Seq<(u64, Seq<u8>)> {
    log.map(|i: int, e: Entry| entry_view(e))
}

pub open spec fn bounded(entries: Seq<Entry>, term: u64) -> bool {
    forall|i: int| 0 <= i < entries.len() ==> #[trigger] entries[i].term <= term
}

pub fn copy_bytes(bytes: &[u8]) -> (result: Vec<u8>)
    ensures result@ == bytes@,
{
    let mut result = Vec::new();
    let mut i = 0;
    while i < bytes.len()
        invariant 0 <= i <= bytes.len(), result@ == bytes@.take(i as int),
        decreases bytes.len() - i,
    {
        result.push(bytes[i]);
        i = i + 1;
        assert(result@ =~= bytes@.take(i as int));
    }
    result
}

pub fn copy_entry(e: &Entry) -> (r: Entry)
    ensures entry_view(r) == entry_view(*e),
{ Entry { term: e.term, data: e.data.clone() } }

pub fn append(log: &mut Vec<Entry>, term: u64, data: &[u8]) -> (ok: bool)
    ensures
        ok == (old(log).len() < usize::MAX),
        ok ==> log.len() == old(log).len() + 1,
        ok ==> log_view(log@) == log_view(old(log)@).push((term, data@)),
        !ok ==> log@ == old(log)@,
        bounded(old(log)@, term) ==> bounded(log@, term),
{
    if log.len() == usize::MAX { return false; }
    log.push(Entry { term, data: copy_bytes(data) });
    assert(log_view(log@) =~= log_view(old(log)@).push((term, data@)));
    true
}

pub fn bytes_equal(a: &[u8], b: &[u8]) -> (same: bool)
    ensures same == (a@ == b@),
{
    if a.len() != b.len() { return false; }
    let mut i = 0;
    while i < a.len()
        invariant a.len() == b.len(), i <= a.len(), forall|j: int| 0 <= j < i ==> a@[j] == b@[j],
        decreases a.len() - i,
    {
        if a[i] != b[i] { return false; }
        i = i + 1;
    }
    assert(a@ =~= b@);
    true
}

pub fn equal(a: &Entry, b: &Entry) -> (same: bool)
    ensures same == (entry_view(*a) == entry_view(*b)),
{
    if a.term != b.term || a.data.len() != b.data.len() { return false; }
    let mut i = 0;
    while i < a.data.len()
        invariant a.term == b.term, a.data.len() == b.data.len(), i <= a.data.len(),
            forall|j: int| 0 <= j < i ==> a.data@[j] == b.data@[j],
        decreases a.data.len() - i,
    {
        if a.data[i] != b.data[i] { return false; }
        i = i + 1;
    }
    assert(a.data@ =~= b.data@);
    true
}

pub fn term_at(log: &Vec<Entry>, base: u64, base_term: u64, index: u64) -> (term: Option<u64>)
    requires base as int + log.len() <= u64::MAX,
    ensures
        index == base ==> term == Some(base_term),
        base < index <= base + log.len() ==> term == Some(log@[(index - base - 1) as int].term),
        index < base || index > base + log.len() ==> term.is_none(),
{
    if index == base { Some(base_term) }
    else if index > base {
        let offset = index - base - 1;
        if offset < log.len() as u64 { Some(log[offset as usize].term) } else { None }
    } else { None }
}

pub fn truncate(log: &mut Vec<Entry>, keep: usize)
    requires keep <= old(log).len(),
    ensures log@ == old(log)@.take(keep as int),
{
    while log.len() > keep
        invariant keep <= log.len() <= old(log).len(), log@ == old(log)@.take(log.len() as int),
        decreases log.len(),
    {
        let ghost before = log@;
        let len = log.len();
        log.pop();
        assert(log.len() == len - 1);
        assert forall|j: int| 0 <= j < log.len() implies log@[j] == old(log)@[j] by {
            assert(log@[j] == before[j]);
            assert(before[j] == old(log)@[j]);
        }
        assert(log@ =~= old(log)@.take(log.len() as int));
    }
}

pub fn compact(log: &mut Vec<Entry>, count: usize)
    requires count <= old(log).len(),
    ensures log.len() == old(log).len() - count,
        log_view(log@) == log_view(old(log)@).skip(count as int),
        forall|limit: u64| #[trigger] bounded(old(log)@, limit) ==> bounded(log@, limit),
{
    let mut remaining = Vec::new();
    let mut i = count;
    while i < log.len()
        invariant count <= i <= log.len(), remaining.len() == i - count,
            log_view(remaining@) == log_view(log@).subrange(count as int, i as int),
        decreases log.len() - i,
    {
        let ghost before = remaining@;
        let e = copy_entry(&log[i]);
        remaining.push(e);
        assert forall|j: int| 0 <= j < remaining.len() implies
            entry_view(remaining@[j]) == entry_view(log@[j + count]) by {
            if j < before.len() {
                assert(log_view(before) == log_view(log@).subrange(count as int, i as int));
                assert(log_view(before)[j] == log_view(log@).subrange(count as int, i as int)[j]);
                assert(entry_view(before[j]) == log_view(log@)[j + count]);
            } else {
                assert(j == before.len());
                assert(entry_view(e) == entry_view(log@[i as int]));
            }
        }
        i = i + 1;
        assert(log_view(remaining@) =~= log_view(log@).subrange(count as int, i as int));
    }
    assert(log_view(remaining@) =~= log_view(log@).skip(count as int));
    *log = remaining;
    assert forall|limit: u64| bounded(old(log)@, limit) implies bounded(log@, limit) by {
        assert forall|j: int| 0 <= j < log.len() implies log@[j].term <= limit by {
            assert(log_view(log@)[j] == log_view(old(log)@).skip(count as int)[j]);
            assert(log_view(old(log)@)[j + count] == entry_view(old(log)@[j + count]));
        }
    }
}

pub fn append_entry(log: &mut Vec<Entry>, entry: &Entry) -> (ok: bool)
    ensures
        ok == (old(log).len() < usize::MAX),
        ok ==> log.len() == old(log).len() + 1,
        ok ==> log_view(log@) == log_view(old(log)@).push(entry_view(*entry)),
        !ok ==> log@ == old(log)@,
{
    append(log, entry.term, &entry.data)
}

// The protected count is the committed portion of this resident suffix.
// A host caller must establish this interpretation; this contract by itself is
// not a proof that the cluster's committed entries are preserved everywhere.
pub fn receive(log: &mut Vec<Entry>, offset: usize, protected: usize, entry: &Entry) -> (ok: bool)
    requires protected <= old(log).len(),
    ensures
        log_view(log@).take(protected as int) == log_view(old(log)@).take(protected as int),
        log.len() >= protected,
        log.len() <= old(log).len() + 1,
        ok ==> log.len() == old(log).len() || log.len() == offset + 1,
        ok ==> log_view(log@) == log_view(old(log)@)
            || log_view(log@) == log_view(old(log)@).take(offset as int).push(entry_view(*entry)),
        ok ==> offset < log.len() && entry_view(log@[offset as int]) == entry_view(*entry)
            && log_view(log@).take(offset as int) == log_view(old(log)@).take(offset as int),
        !ok ==> log@ == old(log)@,
{
    if offset > log.len() { return false; }
    if offset < log.len() {
        if equal(&log[offset], entry) { return true; }
        if offset < protected { return false; }
        truncate(log, offset);
    }
    assert(log.len() == offset);
    assert(log_view(log@) =~= log_view(old(log)@).take(offset as int));
    let ok = append_entry(log, entry);
    if ok {
        assert(log.len() == offset + 1);
        assert(log_view(log@)[offset as int] == entry_view(*entry));
        assert(entry_view(log@[offset as int]) == entry_view(*entry));
    }
    assert(log_view(log@).take(protected as int) =~= log_view(old(log)@).take(protected as int));
    assert(ok ==> log_view(log@).take(offset as int) =~= log_view(old(log)@).take(offset as int));
    ok
}

pub fn suffix(log: &Vec<Entry>, offset: usize) -> (result: Vec<Entry>)
    requires offset <= log.len(),
    ensures log_view(result@) == log_view(log@).skip(offset as int),
{
    let mut result = Vec::new();
    let mut i = offset;
    while i < log.len()
        invariant offset <= i <= log.len(), result.len() == i - offset,
            forall|j: int| 0 <= j < result.len() ==> entry_view(result@[j]) == entry_view(log@[j + offset]),
        decreases log.len() - i,
    {
        result.push(copy_entry(&log[i]));
        i = i + 1;
    }
    assert(log_view(result@) =~= log_view(log@).skip(offset as int));
    result
}

pub proof fn logical_compaction(prefix: Seq<(u64, Seq<u8>)>, resident: Seq<(u64, Seq<u8>)>, count: int)
    requires 0 <= count <= resident.len(),
    ensures prefix + resident == (prefix + resident.take(count)) + resident.skip(count),
{
    assert(resident =~= resident.take(count) + resident.skip(count));
    assert(prefix + resident =~= (prefix + resident.take(count)) + resident.skip(count));
}


pub fn successor(index: u64) -> (r: u64)
    ensures r == if index == u64::MAX { index as int } else { index + 1 },
{ if index == u64::MAX { index } else { index + 1 } }

pub fn valid_terms(entries: &Vec<Entry>, previous: u64, current: u64) -> (ok: bool)
    ensures ok == (forall|i: int| 0 <= i < entries.len() ==> #[trigger] entries@[i].term <= current
        && entries@[i].term >= if i == 0 { previous } else { entries@[i - 1].term }),
{
    let mut i = 0;
    let mut last = previous;
    while i < entries.len()
        invariant i <= entries.len(), last == if i == 0 { previous } else { entries@[i as int - 1].term },
            forall|j: int| 0 <= j < i ==> #[trigger] entries@[j].term <= current
                && entries@[j].term >= if j == 0 { previous } else { entries@[j - 1].term },
        decreases entries.len() - i,
    {
        if entries[i].term > current || entries[i].term < last { return false; }
        last = entries[i].term;
        i = i + 1;
    }
    true
}

pub struct MergeResult { pub success: bool, pub next: u64, pub commit: u64 }

// Full follower log decision, including rejection/backtracking, repair and the
// commit cursor. The surrounding term/identity check is performed by State.
pub fn merge(log: &mut Vec<Entry>, base: u64, base_term: u64, commit: u64,
    prev: u64, prev_term: u64, entries: &Vec<Entry>, leader_commit: u64, current_term: u64) -> (r: MergeResult)
    requires base <= commit <= base as int + old(log).len() <= u64::MAX,
        bounded(old(log)@, current_term),
    ensures
        bounded(log@, current_term),
        base as int + log.len() <= u64::MAX,
        commit <= r.commit <= base as int + log.len(),
        log_view(log@).take((commit - base) as int) == log_view(old(log)@).take((commit - base) as int),
        !r.success ==> r.commit == commit,
        entries.len() == 0 ==> log@ == old(log)@,
        r.success ==> prev >= base && prev as int + entries.len() <= u64::MAX
            && prev as int - base + entries.len() <= log.len()
            && log_view(log@).subrange((prev - base) as int, (prev - base + entries.len()) as int) == log_view(entries@),
{
    let ghost original = log@;
    let original_len = log.len();
    let end = base + log.len() as u64;
    let mut next = successor(end);
    if prev < base { return MergeResult { success: false, next: successor(base), commit }; }
    let found = term_at(log, base, base_term, prev);
    if found != Some(prev_term) {
        if let Some(term) = found {
            let mut first = prev;
            while first > base
                invariant base <= first <= prev <= end, end == base as int + log.len(),
                    log@ == original, original_len == original.len(), original == old(log)@,
                    bounded(log@, current_term),
                decreases first - base,
            {
                if term_at(log, base, base_term, first - 1) != Some(term) { break; }
                first = first - 1;
            }
            next = if first == 0 { 1 } else { first };
        }
        return MergeResult { success: false, next, commit };
    }
    assert(prev <= end);
    let offset = (prev - base) as usize;
    if entries.len() as u64 > u64::MAX - prev || entries.len() > usize::MAX - offset {
        return MergeResult { success: false, next, commit };
    }
    let matched_end = prev + entries.len() as u64;
    if !valid_terms(entries, prev_term, current_term) { return MergeResult { success: false, next, commit }; }
    let protected = (commit - base) as usize;
    let mut i = 0;
    while i < entries.len()
        invariant
            i <= entries.len(), offset == prev - base, base <= commit,
            original == old(log)@, original_len == original.len(), bounded(log@, current_term),
            entries.len() == 0 ==> log@ == original,
            forall|j: int| 0 <= j < entries.len() ==> #[trigger] entries@[j].term <= current_term,
            base as int + original_len <= u64::MAX,
            prev <= base as int + original_len,
            prev as int + entries.len() <= u64::MAX,
            offset as int + entries.len() <= usize::MAX,
            protected == commit - base, protected <= log.len(),
            offset as int + i <= log.len(),
            log.len() <= if original_len > offset + i { original_len as int } else { offset + i },
            base as int + log.len() <= u64::MAX,
            log_view(log@).take(protected as int) == log_view(original).take(protected as int),
            forall|j: int| 0 <= j < i ==> entry_view(log@[offset as int + j]) == entry_view(entries@[j]),
        decreases entries.len() - i,
    {
        let ghost before = log@;
        let ok = receive(log, offset + i, protected, &entries[i]);
        if !ok { return MergeResult { success: false, next, commit }; }
        assert forall|j: int| 0 <= j < log.len() implies log@[j].term <= current_term by {
            if log_view(log@) == log_view(before) {
                assert(log_view(log@)[j] == log_view(before)[j]);
            } else {
                assert(log_view(log@) == log_view(before).take((offset + i) as int).push(entry_view(entries@[i as int])));
                if j < offset + i { assert(log_view(log@)[j] == log_view(before)[j]); }
            }
        }
        assert forall|j: int| 0 <= j < i + 1 implies
            entry_view(log@[offset as int + j]) == entry_view(entries@[j]) by {
            if j < i {
                assert(log_view(log@).take((offset + i) as int)[offset as int + j]
                    == log_view(before).take((offset + i) as int)[offset as int + j]);
                assert(entry_view(before[offset as int + j]) == entry_view(entries@[j]));
            }
        }
        i = i + 1;
    }
    let learned = if leader_commit < matched_end { leader_commit } else { matched_end };
    let new_commit = if commit < learned { learned } else { commit };
    assert(log_view(log@).subrange(offset as int, (offset + entries.len()) as int) =~= log_view(entries@));
    MergeResult { success: true, next: successor(matched_end), commit: new_commit }
}

}
