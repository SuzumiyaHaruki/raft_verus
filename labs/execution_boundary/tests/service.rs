use execution_boundary::{adapter::{self, ACK, PUT}, core::{Core, Peer, ReceivedPut}};
use madsim::{net::{Endpoint, NetSim}, runtime::Handle, time::{sleep, timeout, Duration, Instant}};
use std::net::SocketAddr;

#[test]
fn direct_core_updates_and_constructs_exact_actions() {
    let mut core = Core::new();
    assert_eq!(core.value(), 0);
    for (ip, port, value) in [(2, 9000, 7), (3, 9001, 9), (2, 9000, 0),
        (3, 9001, u64::MAX), (3, 9001, u64::MAX)] {
        let source = Peer { ip, port };
        let action = core.handle(ReceivedPut { source, value });
        assert_eq!(core.value(), value, "actual core memory");
        assert_eq!(action.value, value, "actual constructed Ack");
        assert_eq!(action.destination, source, "actual selected recipient");
    }
}

async fn receive_ack(endpoint: &Endpoint, server: SocketAddr, value: u64) {
    let mut bytes = [0u8; 9];
    let result = timeout(Duration::from_secs(1), endpoint.recv_from(ACK, &mut bytes)).await;
    assert!(result.is_ok(), "Ack was not returned to the requesting client");
    let (length, source) = result.unwrap().unwrap();
    assert_eq!(source, server, "Ack transport source");
    assert_eq!(length, 8, "Ack encoding length");
    assert_eq!(u64::from_le_bytes(bytes[..8].try_into().unwrap()), value,
        "Ack payload must match this client's Put");
}

#[madsim::test]
async fn two_clients_receive_own_ack() {
    scenario(false).await;
}

#[madsim::test]
async fn delayed_old_ack_is_independent_of_new_server_value() {
    scenario(true).await;
}

async fn scenario(delay_first_receive: bool) {
    // Reliable bounded latency is a TEST setting, not a safety assumption.
    NetSim::current().update_config(|c| {
        c.packet_loss_rate = 0.0;
        c.send_latency = Duration::from_millis(1)..Duration::from_millis(2);
    });
    let runtime = Handle::current();
    let server_addr: SocketAddr = "10.0.0.1:9000".parse().unwrap();
    let a_addr: SocketAddr = "10.0.0.2:9000".parse().unwrap();
    let b_addr: SocketAddr = "10.0.0.3:9000".parse().unwrap();
    let server = runtime.create_node().ip(server_addr.ip()).build();
    let a = runtime.create_node().ip(a_addr.ip()).build();
    let b = runtime.create_node().ip(b_addr.ip()).build();
    let ep = server.spawn(async move { Endpoint::bind(server_addr).await.unwrap() }).await.unwrap();
    let ep_a = a.spawn(async move { Endpoint::bind(a_addr).await.unwrap() }).await.unwrap();
    let ep_b = b.spawn(async move { Endpoint::bind(b_addr).await.unwrap() }).await.unwrap();
    let service = server.spawn(adapter::serve(ep, 2));
    let first = a.spawn(async move {
        ep_a.send_to(server_addr, PUT, &7u64.to_le_bytes()).await.unwrap();
        if delay_first_receive { sleep(Duration::from_millis(100)).await; }
        receive_ack(&ep_a, server_addr, 7).await;
        Instant::now()
    });
    let second = b.spawn(async move {
        sleep(Duration::from_millis(20)).await;
        ep_b.send_to(server_addr, PUT, &9u64.to_le_bytes()).await.unwrap();
        receive_ack(&ep_b, server_addr, 9).await;
    });
    let final_state = service.await.unwrap().unwrap();
    let finished = Instant::now();
    assert_eq!(final_state, 9, "second Put is the actual final memory value");
    second.await.unwrap();
    let first_received = first.await.unwrap();
    if delay_first_receive {
        assert!(first_received > finished, "old Ack observed after the new state");
        assert_ne!(7, final_state, "old Ack need not equal current memory");
    }
    println!("A received Ack(7); B received Ack(9); final state=9; delayed={delay_first_receive}");
}
