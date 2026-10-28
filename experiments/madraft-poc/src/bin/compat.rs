//! DRAFT compatibility probe; no Raft protocol behavior claimed.
#[cfg(not(madsim))]
compile_error!("This experiment requires the actual MadSim runtime: --cfg madsim");
use madsim::{
    net::{Endpoint, NetSim},
    runtime::Runtime,
    time::{sleep, timeout, Duration},
};
use raft_verified::{InitError, Node};

fn main() {
    Runtime::with_seed_and_config(17, Default::default()).block_on(async {
        let h = madsim::runtime::Handle::current();
        let a = h.create_node().name("injector").ip("10.0.0.1".parse().unwrap()).build();
        let b = h.create_node().name("follower").ip("10.0.0.2".parse().unwrap()).build();
        let _c = h.create_node().name("third-member").ip("10.0.0.3".parse().unwrap()).build();
        let server = b.spawn(async {
            let ep = Endpoint::bind("10.0.0.2:9000").await.unwrap();
            let mut buf = [0; 32];
            let (n, peer) = ep.recv_from(0, &mut buf).await.unwrap();
            assert_eq!(&buf[..n], b"probe");
            // Invoke the actual existing Verus-backed library, without pretending its seed works.
            assert!(matches!(Node::new(2, vec![]), Err(InitError::InvalidMembership)));
            ep.send_to(peer, 1, b"real-core-called").await.unwrap();
        });
        sleep(Duration::from_millis(20)).await;
        let client = a.spawn(async {
            let ep = Endpoint::bind("10.0.0.1:9000").await.unwrap();
            NetSim::current().update_config(|c| c.packet_loss_rate = 1.0);
            ep.send_to("10.0.0.2:9000", 0, b"lost").await.unwrap();
            let mut buf = [0; 32];
            assert!(timeout(Duration::from_millis(50), ep.recv_from(1, &mut buf)).await.is_err());
            NetSim::current().update_config(|c| {c.packet_loss_rate = 0.0; c.send_latency = Duration::from_millis(3)..Duration::from_millis(4);});
            ep.send_to("10.0.0.2:9000", 0, b"probe").await.unwrap();
            let (n, _) = timeout(Duration::from_secs(1), ep.recv_from(1, &mut buf)).await.unwrap().unwrap();
            assert_eq!(&buf[..n], b"real-core-called");
        });
        client.await.unwrap(); server.await.unwrap();
        h.kill(b.id());
        println!("MADSIM_COMPAT_PASS real_network=true clock=true loss=true kill=true core=raft_verified::Node::new seed=17");
    });
}
