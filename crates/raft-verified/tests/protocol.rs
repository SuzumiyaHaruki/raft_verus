#![cfg(feature = "madraft-poc")]
//! Independent behavior checks of the public event interface; no second Raft engine.
use raft_verified::*;
fn new(id: u64, count: u64) -> Node {
    Node::new(id, (1..=count).collect()).unwrap()
}
fn rpc(sequence: u64) -> OperationId {
    OperationId {
        incarnation: 0,
        sequence,
    }
}
fn request(
    from: u64,
    to: u64,
    term: u64,
    sequence: u64,
    last_index: u64,
    last_term: u64,
) -> Message {
    Message {
        from,
        to,
        term,
        rpc: rpc(sequence),
        body: MessageBody::RequestVote {
            last_index,
            last_term,
        },
    }
}
fn grants(out: &[Effect]) -> usize {
    out.iter()
        .filter(|e| {
            matches!(
                e,
                Effect::Send {
                    message: Message {
                        body: MessageBody::VoteResponse { granted: true }
                            | MessageBody::AppendResponse { accepted: true, .. },
                        ..
                    },
                    ..
                }
            )
        })
        .count()
}
fn write(out: &[Effect]) -> (OperationId, DurableState) {
    out.iter()
        .find_map(|e| match e {
            Effect::Persist { operation, state } => Some((*operation, state.clone())),
            _ => None,
        })
        .expect("required persistence effect")
}
fn complete(n: &mut Node, op: OperationId) -> Vec<Effect> {
    n.handle_event(Event::Complete {
        operation: op,
        kind: CompletionKind::Persist,
        outcome: CompletionOutcome::Success,
    })
    .unwrap()
}
fn timeout(n: &mut Node) -> Vec<Effect> {
    let s = n.observe();
    n.handle_event(Event::ElectionTimeout {
        incarnation: s.incarnation,
        generation: s.deadline,
    })
    .unwrap()
}
fn votes(n: &mut Node) -> Vec<Message> {
    let out = timeout(n);
    assert_eq!(grants(&out), 0);
    let (op, _) = write(&out);
    complete(n, op)
        .into_iter()
        .filter_map(|e| match e {
            Effect::Send { message, .. }
                if matches!(message.body, MessageBody::RequestVote { .. }) =>
            {
                Some(message)
            }
            _ => None,
        })
        .collect()
}
fn vote_reply(q: &Message) -> Message {
    Message {
        from: q.to,
        to: q.from,
        term: q.term,
        rpc: q.rpc,
        body: MessageBody::VoteResponse { granted: true },
    }
}
fn entry(term: u64, value: u8) -> Entry {
    Entry {
        term,
        command: Command::Client {
            request: RequestId {
                client: 7,
                sequence: value as u64,
            },
            bytes: vec![value],
        },
    }
}

#[test]
fn persistence_barrier() {
    let mut n = new(2, 3);
    let out = n
        .handle_event(Event::Receive(request(1, 2, 2, 9, 0, 0)))
        .unwrap();
    assert_eq!(grants(&out), 0);
    let (term_op, term_image) = write(&out);
    assert_eq!((term_image.term, term_image.voted_for), (2, None));
    // The host writing the image is distinct from returning its completion.
    let before = n.observe().clone();
    assert_eq!(
        n.handle_event(Event::Receive(request(1, 2, 2, 9, 0, 0))),
        Err(InputError::Busy)
    );
    assert_eq!(*n.observe(), before);
    let out = complete(&mut n, term_op);
    assert_eq!(grants(&out), 0);
    let (vote_op, vote_image) = write(&out);
    assert_ne!(term_op, vote_op);
    assert_eq!(vote_image.voted_for, Some(1));
    let out = complete(&mut n, vote_op);
    assert_eq!(grants(&out), 1);
    let reply = out
        .iter()
        .find_map(|e| {
            if let Effect::Send { message, .. } = e {
                Some(message)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(
        (reply.from, reply.to, reply.term, reply.rpc),
        (2, 1, 2, rpc(9))
    );
    assert_eq!(
        n.handle_event(Event::Complete {
            operation: vote_op,
            kind: CompletionKind::Persist,
            outcome: CompletionOutcome::Success
        }),
        Err(InputError::UnknownOperation)
    );
}
#[test]
fn duplicate_votes_never_form_quorum() {
    let mut n = new(1, 5);
    let requests = votes(&mut n);
    assert_eq!(requests.len(), 4);
    let q = requests.iter().find(|m| m.to == 2).unwrap();
    for _ in 0..4 {
        let out = n.handle_event(Event::Receive(vote_reply(q))).unwrap();
        assert!(!out.iter().any(|e|matches!(e,Effect::Persist{state,..} if state.log.iter().any(|entry|matches!(entry.command,Command::Noop)))));
        assert_eq!(
            n.observe().role,
            Role::Candidate,
            "duplicate votes formed a quorum"
        );
    }
    let other = requests.iter().find(|m| m.to == 3).unwrap();
    let out = n.handle_event(Event::Receive(vote_reply(other))).unwrap();
    assert_eq!(n.observe().role, Role::Leader);
    let (op, _) = write(&out);
    complete(&mut n, op);
    assert_eq!(n.observe().role, Role::Leader);
}
#[test]
fn stale_vote_responses_do_not_elect() {
    let mut n = new(1, 3);
    let requests = votes(&mut n);
    let q = &requests[0];
    let mut wrong = vote_reply(q);
    wrong.rpc.sequence += 100;
    n.handle_event(Event::Receive(wrong)).unwrap();
    let mut wrong = vote_reply(q);
    wrong.term = 0;
    n.handle_event(Event::Receive(wrong)).unwrap();
    assert_eq!(n.observe().role, Role::Candidate);
    n.handle_event(Event::Receive(vote_reply(q))).unwrap();
    assert_eq!(n.observe().role, Role::Leader);
}
#[test]
fn vote_freshness_and_single_vote_per_term() {
    let mut s = new(2, 3).observe().clone();
    s.hard = DurableState {
        term: 3,
        voted_for: None,
        log: vec![entry(1, 1), entry(3, 2)],
    };
    let mut n = Node::from_observation(s);
    for (from, index, term, grant) in [
        (1, 9, 2, false),
        (1, 1, 3, false),
        (1, 2, 3, true),
        (3, 8, 4, false),
        (1, 2, 3, true),
    ] {
        let out = n
            .handle_event(Event::Receive(request(from, 2, 3, 8, index, term)))
            .unwrap();
        assert_eq!(grants(&out), 0);
        if grant {
            let (op, image) = write(&out);
            assert_eq!(image.voted_for, Some(1));
            assert_eq!(grants(&complete(&mut n, op)), 1);
        } else {
            assert!(!out.iter().any(|e| matches!(e, Effect::Persist { .. })));
            assert!(out.iter().any(|e| matches!(
                e,
                Effect::Send {
                    message: Message {
                        body: MessageBody::VoteResponse { granted: false },
                        ..
                    },
                    ..
                }
            )));
        }
    }
}
#[test]
fn storage_error_and_callback_correlation() {
    for outcome in [CompletionOutcome::NotPerformed, CompletionOutcome::Unknown] {
        let mut n = new(2, 3);
        let out = n
            .handle_event(Event::Receive(request(1, 2, 0, 7, 0, 0)))
            .unwrap();
        let (op, _) = write(&out);
        for (operation, kind, error) in [
            (
                OperationId {
                    incarnation: 1,
                    ..op
                },
                CompletionKind::Persist,
                InputError::StaleCompletion,
            ),
            (
                OperationId { sequence: 99, ..op },
                CompletionKind::Persist,
                InputError::UnknownOperation,
            ),
            (op, CompletionKind::Send, InputError::WrongCompletionKind),
            (op, CompletionKind::Apply, InputError::WrongCompletionKind),
        ] {
            let before = n.observe().clone();
            assert_eq!(
                n.handle_event(Event::Complete {
                    operation,
                    kind,
                    outcome: CompletionOutcome::Success
                }),
                Err(error)
            );
            assert_eq!(*n.observe(), before);
        }
        assert!(n
            .handle_event(Event::Complete {
                operation: op,
                kind: CompletionKind::Persist,
                outcome
            })
            .is_err());
        assert!(n.observe().recovery_required);
        assert_eq!(
            n.handle_event(Event::Receive(request(3, 2, 1, 9, 0, 0))),
            Err(InputError::RecoveryRequired)
        );
    }
}
#[test]
fn append_durable_reply_conflict_and_committed_guard() {
    let mut s = new(2, 3).observe().clone();
    s.hard = DurableState {
        term: 3,
        voted_for: None,
        log: vec![entry(1, 1), entry(2, 2), entry(2, 3)],
    };
    s.commit = 1;
    let mut n = Node::from_observation(s);
    let m = Message {
        from: 1,
        to: 2,
        term: 3,
        rpc: rpc(8),
        body: MessageBody::AppendEntries {
            previous_index: 1,
            previous_term: 1,
            entries: vec![entry(3, 4)],
            leader_commit: 2,
        },
    };
    let out = n.handle_event(Event::Receive(m)).unwrap();
    assert_eq!(grants(&out), 0);
    assert_eq!(n.observe().commit, 1);
    let (op, image) = write(&out);
    assert_eq!(image.log, vec![entry(1, 1), entry(3, 4)]);
    let out = complete(&mut n, op);
    assert_eq!(grants(&out), 1);
    assert_eq!(n.observe().commit, 2);
    assert!(out
        .iter()
        .any(|e| matches!(e,Effect::Deliver{index:1,entry:e,..} if e==&entry(1,1))));
    let before = n.observe().clone();
    let m = Message {
        from: 1,
        to: 2,
        term: 3,
        rpc: rpc(9),
        body: MessageBody::AppendEntries {
            previous_index: 0,
            previous_term: 0,
            entries: vec![entry(3, 5)],
            leader_commit: 2,
        },
    };
    assert_eq!(
        n.handle_event(Event::Receive(m)),
        Err(InputError::MalformedMessage)
    );
    assert_eq!(*n.observe(), before);
}
#[test]
fn recover_saved_image_uses_fresh_identity() {
    let image = DurableState {
        term: 4,
        voted_for: Some(2),
        log: vec![entry(3, 1), entry(4, 2)],
    };
    let mut n = new(2, 3);
    let out = n
        .handle_event(Event::Recover {
            incarnation: 9,
            durable: image.clone(),
            application_cursor: 1,
        })
        .unwrap();
    let (op, saved) = write(&out);
    assert_eq!(saved, image);
    assert_eq!(op.incarnation, 9);
    assert_eq!(grants(&out), 0);
    assert_eq!(n.observe().role, Role::Follower);
    assert_eq!(n.observe().commit, 1);
    assert_eq!(
        n.handle_event(Event::Complete {
            operation: OperationId {
                incarnation: 8,
                sequence: op.sequence
            },
            kind: CompletionKind::Persist,
            outcome: CompletionOutcome::Success
        }),
        Err(InputError::StaleCompletion)
    );
    complete(&mut n, op);
    let before = n.observe().clone();
    assert_eq!(
        n.handle_event(Event::ElectionTimeout {
            incarnation: 8,
            generation: n.observe().deadline
        }),
        Ok(vec![])
    );
    assert_eq!(*n.observe(), before);
    assert_eq!(
        n.handle_event(Event::Recover {
            incarnation: 10,
            durable: image,
            application_cursor: 1
        }),
        Err(InputError::MalformedMessage)
    );
}
#[test]
fn bounded_resources_and_invalid_inputs_are_explicit() {
    for field in ["term", "serial", "deadline"] {
        let mut s = new(1, 3).observe().clone();
        match field {
            "term" => s.hard.term = u64::MAX,
            "serial" => s.serial = u64::MAX,
            _ => s.deadline = u64::MAX,
        };
        let mut n = Node::from_observation(s);
        let before = n.observe().clone();
        let e = Event::ElectionTimeout {
            incarnation: 0,
            generation: n.observe().deadline,
        };
        assert_eq!(n.handle_event(e), Err(InputError::ResourceExhausted));
        assert_eq!(*n.observe(), before);
    }
    let mut n = new(2, 3);
    assert_eq!(
        n.handle_event(Event::Receive(request(9, 2, 0, 0, 0, 0))),
        Err(InputError::UnknownPeer)
    );
    assert_eq!(
        n.handle_event(Event::Receive(request(1, 3, 0, 0, 0, 0))),
        Err(InputError::MalformedMessage)
    );
    assert_eq!(
        n.handle_event(Event::Propose {
            request: RequestId {
                client: 1,
                sequence: 1
            },
            bytes: vec![1]
        }),
        Err(InputError::NotLeader)
    );
}
