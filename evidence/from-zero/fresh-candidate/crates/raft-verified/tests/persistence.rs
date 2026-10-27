//! Independent checks of the same Node coordinator used by the dispatcher.
use raft_verified::*;

fn fresh() -> Node { Node::new(1, vec![1, 2, 3]).unwrap() }
fn image() -> DurableState { DurableState { term: 1, voted_for: Some(1), log: vec![] } }
fn reply() -> Message {
    Message { from: 1, to: 2, term: 1, rpc: OperationId { incarnation: 0, sequence: 42 },
              body: MessageBody::VoteResponse { granted: true } }
}
fn complete(n: &mut Node, operation: OperationId, kind: CompletionKind, outcome: CompletionOutcome)
    -> Result<Option<Message>, InputError> {
    n.step(Event::Complete { operation, kind, outcome })
}

#[test]
fn submission_and_durable_storage_do_not_release_before_callback() {
    let mut n = fresh();
    let mut disk = n.hard.clone();
    let token = n.submit_persist(image(), Some(reply())).unwrap();
    assert_eq!(token, OperationId { incarnation: 0, sequence: 0 });
    assert_eq!(disk.term, 0);
    assert_eq!(n.pending_write.as_ref().unwrap().reply, Some(reply()));
    disk = n.hard.clone(); // Host's atomic durable write, distinct from notification.
    assert_eq!(disk.term, 1);
    assert!(n.pending_write.is_some());
    assert_eq!(complete(&mut n, token, CompletionKind::Persist, CompletionOutcome::Success), Ok(Some(reply())));
    assert!(n.pending_write.is_none());
    assert_eq!(n.hard, disk);
    assert_eq!(complete(&mut n, token, CompletionKind::Persist, CompletionOutcome::Success), Err(InputError::UnknownOperation));
}

#[test]
fn busy_preserves_pending_image_and_fresh_sequence() {
    let mut n = fresh();
    let token = n.submit_persist(image(), None).unwrap();
    let before = n.clone();
    assert_eq!(n.submit_persist(image(), Some(reply())), Err(InputError::Busy));
    assert_eq!(n, before);
    assert_eq!(complete(&mut n, token, CompletionKind::Persist, CompletionOutcome::Success), Ok(None));
    let next = n.submit_persist(image(), None).unwrap();
    assert_eq!(next.sequence, token.sequence + 1);
    assert_eq!(complete(&mut n, token, CompletionKind::Persist, CompletionOutcome::Success), Err(InputError::UnknownOperation));
    assert_eq!(n.pending_write.as_ref().unwrap().operation, next);
}

#[test]
fn incarnation_sequence_and_kind_all_match_before_release() {
    let mut n = fresh();
    let token = n.submit_persist(image(), Some(reply())).unwrap();
    let cases = [
        (OperationId { incarnation: 1, ..token }, CompletionKind::Persist, InputError::StaleCompletion),
        (OperationId { sequence: 1, ..token }, CompletionKind::Persist, InputError::UnknownOperation),
        (token, CompletionKind::Apply, InputError::WrongCompletionKind),
        (token, CompletionKind::Send, InputError::WrongCompletionKind),
    ];
    for (operation, kind, expected) in cases {
        let before = n.clone();
        assert_eq!(complete(&mut n, operation, kind, CompletionOutcome::Success), Err(expected));
        assert_eq!(n, before);
    }
}

#[test]
fn failures_are_fail_stop_and_never_release_the_reply() {
    for (outcome, error) in [(CompletionOutcome::NotPerformed, InputError::StorageFailure),
                             (CompletionOutcome::Unknown, InputError::RecoveryRequired)] {
        let mut n = fresh();
        let token = n.submit_persist(image(), Some(reply())).unwrap();
        assert_eq!(complete(&mut n, token, CompletionKind::Persist, outcome), Err(error));
        assert!(n.recovery_required);
        assert!(n.pending_write.is_none());
        assert_eq!(n.submit_persist(image(), None), Err(InputError::RecoveryRequired));
        assert_eq!(complete(&mut n, token, CompletionKind::Persist, CompletionOutcome::Success), Err(InputError::RecoveryRequired));
    }
}

#[test]
fn exhaustion_is_explicit_and_does_not_wrap_or_modify_state() {
    let mut n = fresh(); n.serial = u64::MAX;
    let before = n.clone();
    assert_eq!(n.submit_persist(image(), None), Err(InputError::ResourceExhausted));
    assert_eq!(n, before);
}

#[test]
fn partial_dispatcher_rejects_every_unimplemented_variant() {
    let mut n = fresh();
    let events = [Event::Receive(reply()), Event::ElectionTimeout { incarnation: 0, generation: 1 },
        Event::HeartbeatTimeout { incarnation: 0, generation: 1 },
        Event::Propose { request: RequestId { client: 1, sequence: 1 }, bytes: vec![1] },
        Event::Recover { incarnation: 1, durable: image(), application_cursor: 0 }];
    for event in events {
        let before = n.clone();
        assert_eq!(n.step(event), Err(InputError::UnsupportedEvent));
        assert_eq!(n, before);
    }
}
