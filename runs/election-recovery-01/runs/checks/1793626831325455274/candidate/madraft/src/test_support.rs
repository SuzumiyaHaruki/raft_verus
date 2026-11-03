//! Host-only MadSim 0.2 test plumbing. No protocol decisions live here.
use madsim::{runtime::{Handle, NodeHandle}, task::NodeId};
use std::net::SocketAddr;

pub fn node(handle: &Handle, addr: SocketAddr) -> NodeHandle {
    handle.get_node(addr.to_string()).expect("test node created before use")
}

pub fn create_node(handle: &Handle, addr: SocketAddr) -> NodeHandle {
    handle.create_node().name(addr.to_string()).ip(addr.ip()).build()
}

pub fn node_id(handle: &Handle, addr: SocketAddr) -> NodeId {
    node(handle, addr).id()
}

pub fn start_node(handle: &Handle, addr: SocketAddr) -> NodeHandle {
    let id = node_id(handle, addr);
    if handle.is_exit(id) {
        handle.restart(id);
    }
    handle.get_node(id).unwrap()
}

pub fn kill(handle: &Handle, addr: SocketAddr) {
    if let Some(node) = handle.get_node(addr.to_string()) {
        handle.kill(node.id());
    }
}

#[madsim::test]
async fn restart_preserves_node_and_simulated_file() {
    use madsim::{fs, time::{sleep, Duration}};
    let handle = Handle::current();
    let addr = "10.0.0.1:1".parse().unwrap();
    let host = create_node(&handle, addr);
    let id = host.id();
    host.spawn(async {
        let f = fs::File::create("probe").await.unwrap();
        f.write_all_at(b"durable", 0).await.unwrap();
        f.sync_all().await.unwrap();
    }).await.unwrap();
    let pending = host.spawn(async { sleep(Duration::from_secs(10)).await; });
    kill(&handle, addr);
    assert!(pending.await.is_err());
    let restarted = start_node(&handle, addr);
    assert_eq!(restarted.id(), id);
    let bytes = restarted.spawn(async { fs::read("probe").await.unwrap() }).await.unwrap();
    assert_eq!(bytes, b"durable");
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Echo(u64);
impl madsim::net::rpc::Request for Echo {
    type Response = u64;
    const ID: u64 = 1000;
}

#[madsim::test]
async fn rpc_partition_and_reconnect() {
    use madsim::{net::{Endpoint, NetSim}, time::Duration};
    let handle = Handle::current();
    let server_addr = "10.0.0.1:1".parse().unwrap();
    let client_addr = "10.0.0.2:1".parse().unwrap();
    let server = create_node(&handle, server_addr);
    let client = create_node(&handle, client_addr);
    let endpoint = server.spawn(async move {
        let ep = Endpoint::bind(server_addr).await.unwrap();
        ep.add_rpc_handler(|Echo(value)| async move { value });
        ep
    }).await.unwrap();
    let request = || async move {
        let ep = Endpoint::bind(client_addr).await.unwrap();
        ep.call_timeout(server_addr, Echo(7), Duration::from_secs(1)).await
    };
    assert_eq!(client.spawn(request()).await.unwrap().unwrap(), 7);
    NetSim::current().disconnect2(server.id(), client.id());
    assert!(client.spawn(request()).await.unwrap().is_err());
    NetSim::current().connect2(server.id(), client.id());
    assert_eq!(client.spawn(request()).await.unwrap().unwrap(), 7);
    drop(endpoint);
}

#[madsim::test]
async fn explicit_abort_preserves_old_drop_cancellation() {
    use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
    use madsim::{task, time::{sleep, Duration}};
    let ran = Arc::new(AtomicBool::new(false));
    let copy = ran.clone();
    let job = task::spawn(async move {
        sleep(Duration::from_secs(1)).await;
        copy.store(true, Ordering::SeqCst);
    });
    job.abort();
    drop(job);
    sleep(Duration::from_secs(2)).await;
    assert!(!ran.load(Ordering::SeqCst));
}
