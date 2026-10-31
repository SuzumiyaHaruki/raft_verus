use async_completion::{adapter::{self, IoControl, Release, ACK, PUT}, core::*};
use async_channel::{Receiver, Sender};
use madsim::{fs, net::{Endpoint, NetSim}, runtime::{Handle, NodeHandle}, time::{timeout, Duration}};
use std::{future::Future, io, net::SocketAddr};

fn put(ip: u32, value: u64) -> Put { Put { source: Peer { ip, port: 9000 }, value } }
fn begin(core: &mut Core, request: Put) -> Write {
    let Begin::Write(w) = core.begin(request) else { panic!("idle must accept") };
    assert_eq!(w.request, request);
    w
}

#[test]
fn core_rejects_busy_mismatched_stale_and_duplicate_completions() {
    let mut core = Core::new();
    assert_eq!(core.complete(Completion { id: 0, success: true }), None);
    let first = begin(&mut core, put(2, 7));
    assert_eq!(core.value(), 0, "acceptance must not complete the value");
    assert_eq!(core.begin(put(3, 9)), Begin::Busy);
    for success in [true, false] {
        assert_eq!(core.complete(Completion { id: first.id + 1, success }), None);
        assert_eq!(core.pending(), Some(first));
        assert_eq!(core.value(), 0);
    }
    assert_eq!(core.complete(Completion { id: first.id, success: false }), None);
    assert_eq!(core.pending(), None);
    let second = begin(&mut core, put(3, 9));
    assert!(second.id > first.id);
    assert_eq!(core.complete(Completion { id: first.id, success: true }), None);
    assert_eq!(core.pending(), Some(second));
    let done = Completion { id: second.id, success: true };
    assert_eq!(core.complete(done), Some(Ack { id: second.id, destination: second.request.source, value: 9 }));
    assert_eq!(core.complete(done), None, "duplicate must not authorize another Ack");
    assert_eq!(core.value(), 9);
    let third = begin(&mut core, put(2, u64::MAX));
    assert_eq!(core.complete(done), None, "old success cannot complete a new Pending");
    assert_eq!(core.pending(), Some(third));
    assert_eq!(core.complete(Completion { id: third.id, success: false }), None);
    assert_eq!(core.value(), 9, "failure retains the prior completed value");
    for value in [u64::MAX, u64::MAX, 0] {
        let w = begin(&mut core, put(2, value));
        assert!(w.id > third.id, "identical client requests get distinct operations");
        assert_eq!(core.complete(Completion { id: w.id, success: true }).unwrap().value, value);
        assert_eq!(core.value(), value);
    }
}

async fn bounded<T>(future: impl Future<Output = T>) -> T {
    timeout(Duration::from_secs(2), future).await.expect("test watchdog: controlled step did not finish")
}

async fn receive_ack(ep: &Endpoint, value: u64) {
    let mut bytes = [0u8; 9];
    let (length, source) = bounded(ep.recv_from(ACK, &mut bytes)).await.unwrap();
    assert_eq!(source, "10.0.0.1:9000".parse::<SocketAddr>().unwrap());
    assert_eq!(length, 8);
    assert_eq!(u64::from_le_bytes(bytes[..8].try_into().unwrap()), value, "original request's Ack");
}

async fn nodes() -> (NodeHandle, NodeHandle, NodeHandle, Endpoint, Endpoint, Endpoint) {
    NetSim::current().update_config(|c| {
        c.packet_loss_rate = 0.0;
        c.send_latency = Duration::from_millis(1)..Duration::from_millis(2);
    });
    let rt = Handle::current();
    let s = rt.create_node().ip("10.0.0.1".parse().unwrap()).build();
    let a = rt.create_node().ip("10.0.0.2".parse().unwrap()).build();
    let b = rt.create_node().ip("10.0.0.3".parse().unwrap()).build();
    let es = s.spawn(Endpoint::bind("10.0.0.1:9000")).await.unwrap().unwrap();
    let ea = a.spawn(Endpoint::bind("10.0.0.2:9000")).await.unwrap().unwrap();
    let eb = b.spawn(Endpoint::bind("10.0.0.3:9000")).await.unwrap().unwrap();
    (s, a, b, es, ea, eb)
}

async fn check_file(server: &NodeHandle, path: &'static str, id: u64, value: u64) {
    // Independent effect oracle: no adapter flag or self-reported record.
    let actual = bounded(server.spawn(fs::read(path))).await.unwrap();
    assert!(actual.is_ok(), "acknowledged write must exist in the actual MadSim file");
    let bytes = actual.unwrap();
    let mut expected = id.to_le_bytes().to_vec();
    expected.extend(value.to_le_bytes());
    assert_eq!(bytes, expected, "actual file must contain THIS operation ID and value");
}

fn control() -> (IoControl, Receiver<Write>, Sender<Release>) {
    let (entered, observer) = async_channel::bounded(1);
    let (release, receiver) = async_channel::bounded(1);
    (IoControl { entered, release: receiver }, observer, release)
}

#[madsim::test]
async fn two_clients_write_and_read_actual_records() {
    let (server, a, b, es, ea, eb) = nodes().await;
    let path = "normal-record";
    assert_eq!(server.spawn(fs::read(path)).await.unwrap().unwrap_err().kind(), io::ErrorKind::NotFound);
    let service = server.spawn(adapter::serve(es, path, 2, None));
    bounded(a.spawn(async move {
        ea.send_to("10.0.0.1:9000", PUT, &7u64.to_le_bytes()).await.unwrap();
        receive_ack(&ea, 7).await;
    })).await.unwrap();
    check_file(&server, path, 0, 7).await; // Before B may overwrite it.
    bounded(b.spawn(async move {
        eb.send_to("10.0.0.1:9000", PUT, &9u64.to_le_bytes()).await.unwrap();
        receive_ack(&eb, 9).await;
    })).await.unwrap();
    check_file(&server, path, 1, 9).await;
    let exit = bounded(service).await.unwrap().unwrap();
    assert!(exit.write_error.is_none());
    assert_eq!(exit.core.value(), 9);
    assert_eq!(exit.core.pending(), None);
    println!("Actual files checked after Ack(7)/op0 and Ack(9)/op1; completed=9");
}

#[madsim::test]
async fn controlled_wait_emits_no_ack_before_actual_write() {
    let (server, a, _b, es, ea, _eb) = nodes().await;
    let path = "delayed-record";
    let (control, entered, release) = control();
    let service = server.spawn(adapter::serve(es, path, 1, Some(control)));
    let (observed, ack) = async_channel::bounded(1);
    let client = a.spawn(async move {
        ea.send_to("10.0.0.1:9000", PUT, &7u64.to_le_bytes()).await.unwrap();
        receive_ack(&ea, 7).await;
        observed.send(()).await.unwrap();
    });
    let waiting = bounded(entered.recv()).await.unwrap();
    assert_eq!((waiting.id, waiting.request.value), (0, 7));
    // Entry is observed first. The gate remains closed for this entire window;
    // this timeout observes absence, it does not guess when I/O should start.
    assert!(timeout(Duration::from_millis(50), ack.recv()).await.is_err(), "Ack before write completion");
    assert_eq!(server.spawn(fs::read(path)).await.unwrap().unwrap_err().kind(), io::ErrorKind::NotFound);
    release.send(Release::Continue).await.unwrap();
    bounded(ack.recv()).await.unwrap();
    bounded(client).await.unwrap();
    check_file(&server, path, 0, 7).await;
    let exit = bounded(service).await.unwrap().unwrap();
    assert!(exit.write_error.is_none());
    assert_eq!(exit.core.value(), 7);
    println!("Gate entered -> no Ack/no file -> release -> real write -> Ack(7)");
}

#[madsim::test]
async fn injected_failure_does_not_ack_or_change_completed_value() {
    let (server, a, b, es, ea, eb) = nodes().await;
    let path = "failure-record";
    let (control, entered, release) = control();
    let service = server.spawn(adapter::serve(es, path, 2, Some(control)));
    let first = a.spawn(async move {
        ea.send_to("10.0.0.1:9000", PUT, &7u64.to_le_bytes()).await.unwrap();
        receive_ack(&ea, 7).await;
    });
    assert_eq!(bounded(entered.recv()).await.unwrap().id, 0);
    release.send(Release::Continue).await.unwrap();
    bounded(first).await.unwrap();
    check_file(&server, path, 0, 7).await;
    let (sent, sent_rx) = async_channel::bounded(1);
    let second = b.spawn(async move {
        eb.send_to("10.0.0.1:9000", PUT, &9u64.to_le_bytes()).await.unwrap();
        let mut bytes = [0u8; 9];
        let packet = eb.recv_from(ACK, &mut bytes).await;
        sent.send(packet).await.unwrap();
    });
    assert_eq!(bounded(entered.recv()).await.unwrap().id, 1);
    release.send(Release::Fail).await.unwrap();
    let exit = bounded(service).await.unwrap().unwrap();
    assert_eq!(exit.write_error.unwrap().to_string(), "injected write failure before create");
    assert_eq!(exit.core.value(), 7);
    assert_eq!(exit.core.pending(), None);
    assert!(timeout(Duration::from_millis(50), sent_rx.recv()).await.is_err(), "failed write authorized an Ack");
    // Only this before-create injection leaves the file unchanged; not a
    // general guarantee about every failed asynchronous write.
    check_file(&server, path, 0, 7).await;
    second.abort();
    println!("Real op0 wrote 7; injected Err for op1 -> no Ack; completed remains 7");
}
