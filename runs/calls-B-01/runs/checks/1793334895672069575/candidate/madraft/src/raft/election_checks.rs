//! Additional recovery checks; the supplied lab tests are unchanged.
use super::*;
use crate::test_support::{create_node, kill, start_node};
use madsim::runtime::Handle;

#[madsim::test]
async fn vote_survives_restart() {
    let runtime = Handle::current();
    let peers: Vec<SocketAddr> = (1..=3).map(|n| SocketAddr::from(([10, 9, 0, n], 1))).collect();
    let addr = peers[0];
    let host = create_node(&runtime, addr);
    let first_peers = peers.clone();
    host.spawn(async move {
        let (r, _) = RaftHandle::new(first_peers, 0).await;
        let reply = r.request_vote(RequestVoteArgs { term: 7, candidate: 1, last_term: 0, last_index: 0 }).await.unwrap();
        assert!(reply.granted);
    }).await.unwrap();
    kill(&runtime, addr);
    let host = start_node(&runtime, addr);
    host.spawn(async move {
        let (r, _) = RaftHandle::new(peers, 0).await;
        assert_eq!(r.term(), 7);
        let no = r.request_vote(RequestVoteArgs { term: 7, candidate: 2, last_term: 0, last_index: 0 }).await.unwrap();
        assert!(!no.granted);
        let repeat = r.request_vote(RequestVoteArgs { term: 7, candidate: 1, last_term: 0, last_index: 0 }).await.unwrap();
        assert!(repeat.granted);
        let newer = r.request_vote(RequestVoteArgs { term: 8, candidate: 2, last_term: 0, last_index: 0 }).await.unwrap();
        assert!(newer.granted);
    }).await.unwrap();
}

#[madsim::test]
async fn incomplete_vote_record_stops_recovery() {
    let runtime = Handle::current();
    let peers: Vec<SocketAddr> = (1..=3).map(|n| SocketAddr::from(([10, 8, 0, n], 1))).collect();
    let addr = peers[0];
    let host = create_node(&runtime, addr);
    let first_peers = peers.clone();
    host.spawn(async move {
        let (r, _) = RaftHandle::new(first_peers, 0).await;
        assert!(r.request_vote(RequestVoteArgs { term: 7, candidate: 1, last_term: 0, last_index: 0 }).await.unwrap().granted);
        // Crash after creating the next record, before writing its value.
        fs::File::create("state.1").await.unwrap();
    }).await.unwrap();
    kill(&runtime, addr);
    let host = start_node(&runtime, addr);
    let error = host.spawn(async move { RaftHandle::try_new(peers, 0).await.err().unwrap().kind() }).await.unwrap();
    assert_eq!(error, io::ErrorKind::InvalidData);
}

#[madsim::test]
async fn exhausted_term_does_not_wrap() {
    let runtime = Handle::current();
    let addr: SocketAddr = "10.7.0.1:1".parse().unwrap();
    let host = create_node(&runtime, addr);
    host.spawn(async move {
        let (r, _) = RaftHandle::new(vec![addr], 0).await;
        let reply = r.request_vote(RequestVoteArgs { term: u64::MAX, candidate: 0, last_term: 0, last_index: 0 }).await.unwrap();
        assert!(reply.granted);
        sleep(Duration::from_secs(2)).await;
        assert_eq!(r.term(), u64::MAX);
        assert!(!r.is_leader());
    }).await.unwrap();
}
