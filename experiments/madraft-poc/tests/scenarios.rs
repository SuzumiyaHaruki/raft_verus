//! Independent expectations. All run() tests use the actual MadSim runtime and transport.
use raft_host_madsim::{request, run, Record, Scenario, Trace};
use raft_verified::{madraft::VoteAction, *};
fn responses(t: &Trace) -> Vec<(u64, u64, bool)> {
    t.records
        .iter()
        .filter_map(|r| match r {
            Record::Delivered(m) => {
                let request = t
                    .records
                    .iter()
                    .find_map(|r| match r {
                        Record::Receive(q) if q.rpc == m.rpc => Some(q),
                        _ => None,
                    })
                    .expect("uncorrelated reply");
                assert_eq!((m.from, m.to), (request.to, request.from));
                match m.body {
                    MessageBody::VoteResponse { granted } => {
                        Some((m.rpc.sequence, m.term, granted))
                    }
                    _ => panic!("wrong reply"),
                }
            }
            _ => None,
        })
        .collect()
}
fn ordered(t: &Trace) {
    for (i, r) in t.records.iter().enumerate() {
        if let Record::Send(m) = r {
            if matches!(m.body, MessageBody::VoteResponse { granted: true }) {
                let completion=t.records[..i].iter().rposition(|r|matches!(r,Record::Complete(_,CompletionOutcome::Success,Ok(Some(reply))) if reply == m)).expect("grant before matching completion");
                let op = match &t.records[completion] {
                    Record::Complete(op, _, _) => op,
                    _ => unreachable!(),
                };
                assert!(t.records[..completion].iter().any(|r|matches!(r,Record::Durable(p,d) if p==op && d.term==m.term && d.voted_for==Some(m.to))));
            }
        }
    }
}
fn emit(t: &Trace) {
    println!("TRACE_BEGIN\n{t:#?}\nTRACE_END");
}
#[test]
fn nominal_exchange() {
    let t = run(Scenario::Nominal, 17);
    emit(&t);
    assert_eq!(responses(&t), vec![(1, 0, true)]);
    assert!(t
        .records
        .iter()
        .any(|r| matches!(r, Record::Timer(s) if s == "Err(UnsupportedEvent)")));
    ordered(&t);
}
#[test]
fn persistence_barrier() {
    let t = run(Scenario::Barrier, 17);
    emit(&t);
    let i = t
        .records
        .iter()
        .position(|r| matches!(r, Record::Checkpoint))
        .unwrap();
    assert!(!t.records[..i]
        .iter()
        .any(|r| matches!(r, Record::Send(_) | Record::Delivered(_))));
    assert!(t.records[..i]
        .iter()
        .any(|r| matches!(r, Record::Durable(_, _))));
    assert!(!t.records[..i]
        .iter()
        .any(|r| matches!(r, Record::Complete(_, _, _))));
    assert_eq!(responses(&t), vec![(1, 0, true)]);
    ordered(&t);
}
#[test]
fn vote_edges() {
    let t = run(Scenario::Edges, 17);
    emit(&t);
    assert_eq!(
        responses(&t),
        vec![
            (1, 0, true),
            (2, 0, true),
            (3, 0, false),
            (4, 2, true),
            (5, 2, false),
            (6, 2, false)
        ]
    );
    ordered(&t);
    assert_eq!(t.final_state.hard.voted_for, Some(3));
    assert_eq!(
        t.records
            .iter()
            .filter(|r| matches!(r, Record::Persist(_, _, true)))
            .count(),
        1
    );
}
#[test]
fn synthetic_log_boundary() {
    let t = run(Scenario::BoundaryLog, 17);
    emit(&t);
    assert_eq!(
        responses(&t),
        vec![(1, 2, false), (2, 2, false), (3, 2, true)]
    );
    ordered(&t);
}
#[test]
fn fault_loss_duplicate_delay_repeats() {
    let a = run(Scenario::Faults, 771);
    let b = run(Scenario::Faults, 771);
    emit(&a);
    emit(&b);
    assert_eq!(a, b, "fixed seed must reproduce normalized protocol trace");
    assert_eq!(responses(&a), vec![(1, 0, true)]);
    let i = a
        .records
        .iter()
        .position(|r| matches!(r, Record::Checkpoint))
        .unwrap();
    assert!(!a.records[..i]
        .iter()
        .any(|r| matches!(r, Record::Receive(_) | Record::Delivered(_))));
    assert!(a
        .records
        .iter()
        .any(|r| matches!(r,Record::Decision(s,_) if s.contains("Busy"))));
    ordered(&a);
}
#[test]
fn fault_crash_after_durable_before_callback() {
    let t = run(Scenario::Crash, 17);
    emit(&t);
    assert!(t.records.iter().any(|r| matches!(r, Record::Crash)));
    assert!(!t
        .records
        .iter()
        .any(|r| matches!(r, Record::Complete(_, _, _) | Record::Send(_))));
    assert_eq!(t.disk.unwrap().voted_for, Some(1));
    assert!(responses(&Trace { disk: None, ..t }).is_empty());
}
#[test]
fn fault_storage_failure_and_unknown_never_grant() {
    for s in [Scenario::Failed, Scenario::Unknown] {
        let t = run(s, 17);
        emit(&t);
        assert!(responses(&t).is_empty());
        assert!(t.final_state.recovery_required);
        if s == Scenario::Failed {
            assert_eq!(t.disk.unwrap().voted_for, None);
        }
    }
}
#[test]
fn core_boundary_errors_and_token_correlation() {
    assert!(Node::feasibility_follower(2, vec![]).is_err());
    assert!(Node::feasibility_follower(2, vec![1, 2, 1]).is_err());
    assert!(Node::feasibility_follower(9, vec![1, 2, 3]).is_err());
    let mut n = Node::feasibility_follower(2, vec![1, 2, 3]).unwrap();
    assert_eq!(
        n.handle_request_vote(request(9, 0, 0, 0, 1)),
        Err(InputError::UnknownPeer)
    );
    let mut bad = request(1, 0, 0, 0, 1);
    bad.to = 3;
    assert_eq!(
        n.handle_request_vote(bad),
        Err(InputError::MalformedMessage)
    );
    let op = match n.handle_request_vote(request(1, 0, 0, 0, 1)).unwrap() {
        VoteAction::Persist { operation, .. } => operation,
        _ => panic!(),
    };
    for (p, k, error) in [
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
    ] {
        let before = n.observe().clone();
        assert_eq!(n.finish_vote(p, k, CompletionOutcome::Success), Err(error));
        assert_eq!(*n.observe(), before);
    }
    assert!(n
        .finish_vote(op, CompletionKind::Persist, CompletionOutcome::Success)
        .unwrap()
        .is_some());
    assert_eq!(
        n.finish_vote(op, CompletionKind::Persist, CompletionOutcome::Success),
        Err(InputError::UnknownOperation)
    );
    let term_op = match n.handle_request_vote(request(3, 2, 0, 0, 2)).unwrap() {
        VoteAction::Persist { operation, retry } => {
            assert!(retry);
            operation
        }
        _ => panic!(),
    };
    assert_eq!(
        n.finish_vote(op, CompletionKind::Persist, CompletionOutcome::Success),
        Err(InputError::UnknownOperation)
    );
    assert_eq!(
        n.finish_vote(term_op, CompletionKind::Persist, CompletionOutcome::Success),
        Ok(None)
    );
    let mut boundary = n.observe().clone();
    boundary.serial = u64::MAX;
    n = Node::from_observation(boundary);
    assert_eq!(
        n.handle_request_vote(request(3, 2, 0, 0, 3)),
        Err(InputError::ResourceExhausted)
    );
}
