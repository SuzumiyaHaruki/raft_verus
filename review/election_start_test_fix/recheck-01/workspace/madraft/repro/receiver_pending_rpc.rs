//! Supplemental actual-host diagnostic. The protected suite remains unchanged.
//! This covers the RPC suffix unreachable after its closed-channel assertion.
#[macro_use]
extern crate log;
include!("../src/raft/raft.rs");

fn diagnostic_peers() -> Vec<SocketAddr> {
    ["10.0.0.1:9000", "10.0.0.2:9000", "10.0.0.3:9000"]
        .iter().map(|s| s.parse().unwrap()).collect()
}
async fn diagnostic_call(node: &madsim::runtime::NodeHandle, endpoint: &net::Endpoint, candidate: u64)
    -> std::result::Result<RequestVoteReply, String>
{
    let ep = endpoint.clone();
    timeout(Duration::from_secs(3), node.spawn(async move {
        ep.call_timeout::<RequestVoteArgs>(diagnostic_peers()[0], RequestVoteArgs {
            term: 1, candidate_id: candidate, last_log_term: 0, last_log_index: 0,
        }, Duration::from_secs(2)).await.unwrap()
    })).await.unwrap().unwrap()
}
async fn diagnostic_file(node: &madsim::runtime::NodeHandle, expected: [u64; 6]) {
    let bytes = node.spawn(fs::read("state")).await.unwrap().unwrap();
    let expected: Vec<u8> = expected.iter().flat_map(|w| w.to_le_bytes()).collect();
    assert_eq!(bytes, expected);
}
fn main() {
    for seed in 1..=5 {
        madsim::runtime::Runtime::with_seed_and_config(seed, Default::default()).block_on(async move {
            net::NetSim::current().update_config(|c| {
                c.packet_loss_rate = 0.;
                c.send_latency = Duration::from_millis(1)..Duration::from_millis(2);
            });
            let rt = madsim::runtime::Handle::current();
            let server = rt.create_node().ip(diagnostic_peers()[0].ip()).build();
            let client = rt.create_node().ip(diagnostic_peers()[1].ip()).build();
            let other = rt.create_node().ip(diagnostic_peers()[2].ip()).build();
            let ep = client.spawn(net::Endpoint::bind(diagnostic_peers()[1])).await.unwrap().unwrap();
            let other_ep = other.spawn(net::Endpoint::bind(diagnostic_peers()[2])).await.unwrap().unwrap();
            let h = server.spawn(async { RaftHandle::new(diagnostic_peers(), 0).await.0 }).await.unwrap();
            let (entered, waiting) = async_channel::bounded(1);
            let (release, released) = async_channel::bounded(1);
            h.vote_hooks.lock().unwrap().barrier = Some(vote_host::Barrier {
                op: 0, after_write: true, entered, release: released,
            });
            let (done, observed) = async_channel::bounded(1);
            let c = client.clone();
            let e = ep.clone();
            let running = task::spawn(async move {
                done.send(diagnostic_call(&c, &e, 1).await).await.unwrap();
            });
            let w = timeout(Duration::from_secs(3), waiting.recv()).await.unwrap().unwrap();
            assert_eq!(h.inner.lock().unwrap().vote.inspect().pending, Some(w));
            assert!(timeout(Duration::from_millis(30), observed.recv()).await.is_err());
            diagnostic_file(&server, [0, 1, 1, 1, 0, 0]).await;
            let old = Arc::downgrade(&h.inner);
            let id = server.id();
            rt.kill(id);
            running.abort();
            assert!(timeout(Duration::from_secs(3), running).await.unwrap().is_err());
            drop(h);
            drop(server);
            drop(release);
            timeout(Duration::from_secs(3), async {
                while old.upgrade().is_some() { task::yield_now().await; }
            }).await.unwrap();
            assert!(observed.is_closed() && observed.is_empty());
            assert!(matches!(timeout(Duration::from_millis(30), observed.recv()).await, Ok(Err(_))));
            rt.restart(id);
            let restarted = rt.get_node(id).unwrap();
            diagnostic_file(&restarted, [0, 1, 1, 1, 0, 0]).await;
            let fresh = restarted.spawn(async {
                RaftHandle::new_receiver(diagnostic_peers(), 0, true).await.0
            }).await.unwrap();
            let v = fresh.inner.lock().unwrap().vote.inspect();
            assert_eq!((v.state.term, v.state.voted_for, v.next, v.pending, v.role),
                (1, Some(1), 1, None, vote_api::Role::Follower));
            assert_eq!(fresh.inner.lock().unwrap().vote.complete(vote_api::Completion {
                op: 0, success: true,
            }), None);
            assert!(!diagnostic_call(&other, &other_ep, 2).await.unwrap().vote_granted);
            diagnostic_file(&restarted, [1, 1, 1, 1, 0, 0]).await;
            assert!(diagnostic_call(&client, &ep, 1).await.unwrap().vote_granted);
            diagnostic_file(&restarted, [2, 1, 1, 1, 0, 0]).await;
            println!("seed {seed}: actual pending-write restart, closed/no-message observer, stale completion, rival denial, chosen-candidate retry and fresh file IDs PASS");
            rt.kill(id);
            rt.kill(client.id());
            rt.kill(other.id());
        });
    }
}
