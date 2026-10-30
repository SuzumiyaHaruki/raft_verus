use super::*;
use crate::test_support::{create_node, start_node, kill};
use futures::StreamExt;
use madsim::runtime::Handle;

#[madsim::test]
async fn durable_vote_survives_each_write_boundary() {
    let runtime = Handle::current();
    for stage in 0..8 {
        let address: SocketAddr = ([10, 250, 1, stage as u8 + 1], 1).into();
        let other: SocketAddr = ([10, 250, 2, stage as u8 + 1], 1).into();
        let node = create_node(&runtime, address);
        let peers = vec![address, other];
        let initial_peers = peers.clone();
        let raft = node.spawn(async move {
            let (raft, _) = RaftHandle::new(initial_peers, 0).await;
            let reply = raft.request_vote(RequestVoteArgs { term: 7, candidate: 0, last: 0, last_term: 0 }).await.unwrap();
            assert!(reply.granted);
            raft
        }).await.unwrap();
        let (reached, mut boundary) = mpsc::unbounded();
        raft.inner.lock().unwrap().persist_pause = Some((stage, reached));
        let pending = node.spawn(async move {
            raft.request_vote(RequestVoteArgs { term: 8, candidate: 0, last: 0, last_term: 0 }).await
        });
        boundary.next().await.expect("write boundary reached");
        kill(&runtime, address);
        assert!(pending.await.is_err());
        let restarted = start_node(&runtime, address);
        restarted.spawn(async move {
            let (raft, _) = RaftHandle::new(peers, 0).await;
            assert!(raft.term() >= 7);
            let reply = raft.request_vote(RequestVoteArgs { term: 7, candidate: 1, last: 0, last_term: 0 }).await.unwrap();
            assert!(!reply.granted, "previously acknowledged ballot changed after stage {}", stage);
        }).await.unwrap();
        kill(&runtime, address);
    }
}

#[madsim::test]
async fn snapshot_survives_each_write_boundary() {
    let runtime = Handle::current();
    for stage in 0..8 {
        let address: SocketAddr = ([10, 251, 1, stage as u8 + 1], 1).into();
        let node = create_node(&runtime, address);
        let raft = node.spawn(async move {
            let (raft, mut applied) = RaftHandle::new(vec![address], 0).await;
            sleep(Duration::from_millis(800)).await;
            assert!(raft.is_leader());
            let start = raft.start(b"committed application state").await.unwrap();
            match applied.next().await.unwrap() {
                ApplyMsg::Command { data, index } => {
                    assert_eq!(index, start.index);
                    raft.snapshot(index, &data).await.unwrap();
                }
                _ => panic!("expected first committed command"),
            }
            raft
        }).await.unwrap();
        let (reached, mut boundary) = mpsc::unbounded();
        raft.inner.lock().unwrap().persist_pause = Some((stage, reached));
        let pending = node.spawn(async move { raft.start(b"unacknowledged suffix").await });
        boundary.next().await.expect("write boundary reached");
        kill(&runtime, address);
        assert!(pending.await.is_err());
        let restarted = start_node(&runtime, address);
        restarted.spawn(async move {
            let (raft, mut applied) = RaftHandle::new(vec![address], 0).await;
            match applied.next().await.unwrap() {
                ApplyMsg::Snapshot { data, term, index } => {
                    assert_eq!(index, 1);
                    assert_eq!(data, b"committed application state");
                    assert!(raft.cond_install_snapshot(term, index, &data).await);
                }
                _ => panic!("expected recovered committed snapshot"),
            }
        }).await.unwrap();
        kill(&runtime, address);
    }
}
