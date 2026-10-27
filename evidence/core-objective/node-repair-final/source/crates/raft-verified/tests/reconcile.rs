//! DRAFT independent runtime regression cases for the shared executable slice.
use raft_verified::{reconcile_log, Command, Entry, ReconcileError, RequestId};

fn entries(terms: &[u64]) -> Vec<Entry> {
    terms
        .iter()
        .map(|&term| Entry {
            term,
            command: Command::Noop,
        })
        .collect()
}

#[test]
fn empty_append_preserves_matching_tail() {
    let mut log = entries(&[1, 2, 3]);
    let before = log.clone();
    assert_eq!(reconcile_log(&mut log, 1, vec![]), Ok(()));
    assert_eq!(log, before);
}

#[test]
fn shorter_matching_input_preserves_tail() {
    let mut log = entries(&[1, 2, 3, 4]);
    assert_eq!(reconcile_log(&mut log, 1, entries(&[2, 3])), Ok(()));
    assert_eq!(log, entries(&[1, 2, 3, 4]));
}

#[test]
fn all_matching_entries_are_unchanged() {
    let mut log = entries(&[1, 2, 3]);
    assert_eq!(reconcile_log(&mut log, 0, entries(&[1, 2, 3])), Ok(()));
    assert_eq!(log, entries(&[1, 2, 3]));
}

#[test]
fn replaces_from_first_conflict_only() {
    let mut log = entries(&[1, 2, 3, 3]);
    assert_eq!(reconcile_log(&mut log, 1, entries(&[2, 4, 5])), Ok(()));
    assert_eq!(log, entries(&[1, 2, 4, 5]));
}

#[test]
fn appends_missing_suffix_after_matching_entries() {
    let mut log = entries(&[1, 2]);
    assert_eq!(reconcile_log(&mut log, 1, entries(&[2, 3, 4])), Ok(()));
    assert_eq!(log, entries(&[1, 2, 3, 4]));
}

#[test]
fn appends_at_log_end_and_initializes_empty_log() {
    let mut log = vec![];
    assert_eq!(reconcile_log(&mut log, 0, entries(&[1])), Ok(()));
    assert_eq!(reconcile_log(&mut log, 1, entries(&[2])), Ok(()));
    assert_eq!(log, entries(&[1, 2]));
}

#[test]
fn first_entry_can_conflict_and_suffix_can_shrink() {
    let mut log = entries(&[1, 2, 3]);
    assert_eq!(reconcile_log(&mut log, 0, entries(&[9])), Ok(()));
    assert_eq!(log, entries(&[9]));
}

#[test]
fn invalid_predecessors_leave_log_unchanged() {
    for previous in [4, usize::MAX] {
        let mut log = entries(&[1, 2, 3]);
        assert_eq!(
            reconcile_log(&mut log, previous, entries(&[9])),
            Err(ReconcileError::InvalidIndex)
        );
        assert_eq!(log, entries(&[1, 2, 3]));
    }
}

#[test]
fn zero_and_end_predecessors_accept_empty_input() {
    for previous in [0, 3] {
        let mut log = entries(&[1, 2, 3]);
        assert_eq!(reconcile_log(&mut log, previous, vec![]), Ok(()));
        assert_eq!(log, entries(&[1, 2, 3]));
    }
    assert_eq!(reconcile_log(&mut vec![], 0, vec![]), Ok(()));
}

#[test]
fn matching_term_retains_existing_command_as_model_requires() {
    let old = Entry {
        term: 7,
        command: Command::Client {
            request: RequestId {
                client: 1,
                sequence: 2,
            },
            bytes: vec![10, 20],
        },
    };
    let mut log = vec![old.clone()];
    assert_eq!(reconcile_log(&mut log, 0, entries(&[7])), Ok(()));
    assert_eq!(log, vec![old]);
}

#[test]
fn conflict_copies_complete_incoming_command() {
    let incoming = Entry {
        term: 7,
        command: Command::Client {
            request: RequestId {
                client: 10,
                sequence: 20,
            },
            bytes: vec![1, 2, 3],
        },
    };
    let mut log = entries(&[1]);
    assert_eq!(reconcile_log(&mut log, 0, vec![incoming.clone()]), Ok(()));
    assert_eq!(log, vec![incoming]);
}
