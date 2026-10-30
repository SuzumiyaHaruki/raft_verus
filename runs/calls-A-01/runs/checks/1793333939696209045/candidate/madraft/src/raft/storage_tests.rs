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

// All snapshots and intervening commands originate in a real elected leader.
// Only the service consumer is delayed; RPCs and Raft tasks continue running.
#[madsim::test]
async fn delayed_snapshot_consumer_keeps_ordered_prefix() {
    let runtime = Handle::current();
    let peers: Vec<SocketAddr> = (1..=3).map(|i| ([10, 252, 0, i], 1).into()).collect();
    let mut nodes = vec![];
    let mut rafts = vec![];
    let mut channels = vec![];
    for me in 0..3 {
        let node = create_node(&runtime, peers[me]);
        let addresses = peers.clone();
        let (raft, channel) = node.spawn(RaftHandle::new(addresses, me)).await.unwrap();
        nodes.push(node); rafts.push(raft); channels.push(Some(channel));
    }
    sleep(Duration::from_millis(800)).await;
    let leader = rafts.iter().position(|r| r.is_leader()).expect("elected leader");
    let victim = (leader + 1) % 3;
    let victim_id = crate::test_support::node_id(&runtime, peers[victim]);
    net::NetSim::current().disconnect(victim_id);
    let h = rafts[leader].clone();
    nodes[leader].spawn(async move { h.start(b"one").await.unwrap(); }).await.unwrap();
    assert!(matches!(channels[leader].as_mut().unwrap().next().await.unwrap(), ApplyMsg::Command { index: 1, .. }));
    let h = rafts[leader].clone();
    nodes[leader].spawn(async move { h.snapshot(1, b"one").await.unwrap(); }).await.unwrap();
    net::NetSim::current().connect(victim_id);
    while rafts[victim].inner.lock().unwrap().disk.base < 1 { sleep(Duration::from_millis(10)).await; }
    let h = rafts[leader].clone();
    nodes[leader].spawn(async move { h.start(b"two").await.unwrap(); }).await.unwrap();
    while rafts[victim].inner.lock().unwrap().applied < 2 { sleep(Duration::from_millis(10)).await; }
    net::NetSim::current().disconnect(victim_id);
    let h = rafts[leader].clone();
    nodes[leader].spawn(async move { h.start(b"three").await.unwrap(); }).await.unwrap();
    for expected in 2..=3 {
        match channels[leader].as_mut().unwrap().next().await.unwrap() {
            ApplyMsg::Command { index, .. } => assert_eq!(index, expected),
            _ => panic!("leader application order"),
        }
    }
    let h = rafts[leader].clone();
    nodes[leader].spawn(async move { h.snapshot(3, b"three").await.unwrap(); }).await.unwrap();
    net::NetSim::current().connect(victim_id);
    while rafts[victim].inner.lock().unwrap().disk.base < 3 { sleep(Duration::from_millis(10)).await; }
    let mut applied = 0;
    let mut channel = channels[victim].take().unwrap();
    while applied < 3 {
        match channel.next().await.unwrap() {
            ApplyMsg::Snapshot { data, term, index } => {
                let h = rafts[victim].clone();
                if nodes[victim].spawn(async move { h.cond_install_snapshot(term, index, &data).await }).await.unwrap() { applied = applied.max(index); }
            }
            ApplyMsg::Command { index, .. } => {
                assert_eq!(index, applied + 1, "queued snapshot was incorrectly invalidated");
                applied = index;
            }
        }
    }
}
