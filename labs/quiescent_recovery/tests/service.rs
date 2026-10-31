use quiescent_recovery::{adapter::{self, Hooks, IoControl, Mode, View, ACK, PUT}, core::*};
use async_channel::{Receiver, Sender};
use madsim::{fs, net::{Endpoint, NetSim}, runtime::{Handle, NodeHandle}, time::{timeout, Duration}};
use std::{future::Future, io, net::SocketAddr};

fn put(value: u64) -> Put { Put { source: Peer { ip: 2, port: 9000 }, value } }
fn write(core: &mut Core, request: Put) -> Write {
    let Begin::Write(w) = core.begin(request) else { panic!("idle must accept") };
    assert_eq!(w.request, request);
    w
}

#[test]
fn recovered_core_preserves_rules_and_rejects_old_notifications() {
    let mut core = Core::recover(Record { id: 1, value: 9 }).unwrap();
    assert_eq!((core.value(), core.next_id(), core.pending()), (9, 2, None));
    for id in [0, 1] {
        for success in [false, true] {
            assert_eq!(core.complete(Completion { id, success }), None);
            assert_eq!((core.value(), core.next_id(), core.pending()), (9, 2, None));
        }
    }
    let w = write(&mut core, put(11));
    assert_eq!((w.id, core.value()), (2, 9));
    assert_eq!(core.begin(put(99)), Begin::Busy);
    for id in [0, 1, 3, u64::MAX] {
        for success in [false, true] {
            assert_eq!(core.complete(Completion { id, success }), None);
            assert_eq!((core.value(), core.pending()), (9, Some(w)));
        }
    }
    let done = Completion { id: w.id, success: true };
    assert_eq!(core.complete(done), Some(Ack { id: 2, destination: w.request.source, value: 11 }));
    assert_eq!(core.complete(done), None);
    let fail = write(&mut core, put(13));
    assert_eq!(core.complete(Completion { id: fail.id, success: false }), None);
    assert_eq!((core.value(), core.next_id(), core.pending()), (11, 4, None));
    // Empty Pending after failure is NOT the allowed restart point: last
    // successful record is ID 2, but ID 3 has already been consumed.
    assert!(core.next_id() > done.id + 1);
    let next = write(&mut core, put(u64::MAX));
    assert_eq!(next.id, 4);
    assert_eq!(core.complete(Completion { id: 4, success: true }).unwrap().value, u64::MAX);
}

#[test]
fn recovery_id_boundaries_are_checked_without_fake_history() {
    assert!(Core::recover(Record { id: u64::MAX, value: 9 }).is_none());
    // Actual checked constructor, not manual field/ghost-history injection.
    let mut last = Core::recover(Record { id: u64::MAX - 1, value: u64::MAX }).unwrap();
    assert_eq!((last.value(), last.next_id(), last.pending()), (u64::MAX, u64::MAX, None));
    assert_eq!(last.begin(put(1)), Begin::Exhausted);
    assert_eq!(last.begin(put(1)), Begin::Exhausted);
    assert_eq!(last.complete(Completion { id: u64::MAX - 1, success: true }), None);
    let mut penultimate = Core::recover(Record { id: u64::MAX - 2, value: 0 }).unwrap();
    let w = write(&mut penultimate, put(7));
    assert_eq!(w.id, u64::MAX - 1);
    assert_eq!(penultimate.complete(Completion { id: w.id, success: true }).unwrap().value, 7);
    assert_eq!(penultimate.begin(put(8)), Begin::Exhausted);
}

async fn bounded<T>(f: impl Future<Output = T>) -> T {
    timeout(Duration::from_secs(2), f).await.expect("test watchdog: controlled step did not finish")
}
fn server_address() -> SocketAddr { "10.0.0.1:9000".parse().unwrap() }
fn hooks() -> (Hooks, Receiver<View>, Receiver<View>) {
    let (started, start) = async_channel::bounded(1);
    let (parked, park) = async_channel::bounded(1);
    (Hooks { started, parked }, start, park)
}
async fn ack(ep: &Endpoint, expected: u64) {
    let mut bytes = [0u8; 9];
    let (n, source) = bounded(ep.recv_from(ACK, &mut bytes)).await.unwrap();
    assert_eq!((n, source), (8, server_address()));
    assert_eq!(u64::from_le_bytes(bytes[..8].try_into().unwrap()), expected);
}
async fn request(client: &NodeHandle, ep: &Endpoint, value: u64) {
    let ep = ep.clone();
    bounded(client.spawn(async move {
        ep.send_to(server_address(), PUT, &value.to_le_bytes()).await.unwrap();
        ack(&ep, value).await;
    })).await.unwrap();
}
async fn check_file(node: &NodeHandle, path: &'static str, id: u64, value: u64) {
    let bytes = bounded(node.spawn(fs::read(path))).await.unwrap().unwrap();
    let mut expected = id.to_le_bytes().to_vec();
    expected.extend(value.to_le_bytes());
    assert_eq!(bytes, expected, "independent actual-file ID/value oracle");
}
struct Dropped(Sender<()>);
impl Drop for Dropped { fn drop(&mut self) { self.0.try_send(()).unwrap(); } }

#[madsim::test]
async fn confirmed_writes_kill_restart_and_continue_from_real_file() {
    NetSim::current().update_config(|c| {
        c.packet_loss_rate = 0.0;
        c.send_latency = Duration::from_millis(1)..Duration::from_millis(2);
    });
    let rt = Handle::current();
    let server = rt.create_node().ip(server_address().ip()).build();
    let server_id = server.id();
    let a = rt.create_node().ip("10.0.0.2".parse().unwrap()).build();
    let b = rt.create_node().ip("10.0.0.3".parse().unwrap()).build();
    let ea = a.spawn(Endpoint::bind("10.0.0.2:9000")).await.unwrap().unwrap();
    let eb = b.spawn(Endpoint::bind("10.0.0.3:9000")).await.unwrap().unwrap();
    let path = "confirmed-record";
    let (first_hooks, first_start, first_park) = hooks();
    let (dropped, old_dropped) = async_channel::bounded(1);
    let first = server.spawn(async move {
        let _drop_notice = Dropped(dropped);
        adapter::serve(server_address(), path, Mode::Fresh, 2, first_hooks, None).await
    });
    let initial = bounded(first_start.recv()).await.unwrap();
    assert_eq!((initial.value, initial.next, initial.pending), (0, 0, false));
    request(&a, &ea, 7).await;
    check_file(&server, path, 0, 7).await;
    request(&b, &eb, 9).await; // The client has RECEIVED the final Ack.
    let parked = bounded(first_park.recv()).await.unwrap();
    assert_eq!((parked.value, parked.next, parked.pending), (9, 2, false));
    check_file(&server, path, 1, 9).await;
    // Count-limited adapter is now parked: no later operation can be accepted.
    rt.kill(server_id);
    assert!(rt.is_exit(server_id));
    let stopped = bounded(first).await;
    assert!(stopped.is_err() && stopped.unwrap_err().is_cancelled());
    bounded(old_dropped.recv()).await.unwrap();
    drop(server); // Do not use its old, killed spawner after restart.
    rt.restart(server_id);
    let restarted = rt.get_node(server_id).unwrap();
    assert_eq!(restarted.id(), server_id);
    assert!(!rt.is_exit(server_id));
    check_file(&restarted, path, 1, 9).await; // No copy to a new node.

    let (second_hooks, second_start, second_park) = hooks();
    let (entered, waiting) = async_channel::bounded(1);
    let (release, released) = async_channel::bounded(1);
    // Only path/mode/control channels cross into the new service, never old
    // Core, saved value, saved next, or the test's expected record.
    let second = restarted.spawn(adapter::serve(server_address(), path, Mode::Recover, 1,
        second_hooks, Some(IoControl { entered, release: released })));
    let restored = bounded(second_start.recv()).await.unwrap();
    assert_eq!(restored.value, 9, "recovery must restore the file's completed value");
    assert_eq!(restored.next, 2, "recovery must resume after the last accepted ID");
    assert!(!restored.pending);

    let (observed, observed_ack) = async_channel::bounded(1);
    let receiver = ea.clone();
    let listening = a.spawn(async move { ack(&receiver, 11).await; observed.send(()).await.unwrap(); });
    assert!(timeout(Duration::from_millis(50), observed_ack.recv()).await.is_err(), "recovery emitted an unsolicited Ack");
    let sender = ea.clone();
    bounded(a.spawn(async move { sender.send_to(server_address(), PUT, &11u64.to_le_bytes()).await.unwrap(); })).await.unwrap();
    let (write, during) = bounded(waiting.recv()).await.unwrap();
    assert_eq!((write.id, write.request.value, during.value, during.next, during.pending), (2, 11, 9, 3, true));
    assert!(timeout(Duration::from_millis(50), observed_ack.recv()).await.is_err(), "Ack before the controlled write completes");
    check_file(&restarted, path, 1, 9).await;
    release.send(()).await.unwrap();
    bounded(observed_ack.recv()).await.unwrap();
    bounded(listening).await.unwrap();
    let final_state = bounded(second_park.recv()).await.unwrap();
    assert_eq!((final_state.value, final_state.next, final_state.pending), (11, 3, false));
    check_file(&restarted, path, 2, 11).await;
    rt.kill(server_id); // Cleanup only; no second recovery is claimed.
    assert!(bounded(second).await.unwrap_err().is_cancelled());
    println!("Ack(7)/op0 -> Ack(9)/op1 -> parked -> kill/drop -> same-node restart -> file op1/9 -> recover 9,next2 -> controlled op2 -> Ack(11),file op2/11");
}

#[madsim::test]
async fn recovery_mode_rejects_missing_bad_length_and_illegal_id() {
    let node = Handle::current().create_node().build();
    bounded(node.spawn(async {
        let path = "boundary-record";
        let missing = adapter::load_core(path, Mode::Recover).await.err().unwrap();
        assert_eq!(missing.kind(), io::ErrorKind::NotFound);
        assert_eq!(adapter::load_core(path, Mode::Fresh).await.unwrap().value(), 0);
        for length in [0, 15, 17] {
            let file = fs::File::create(path).await.unwrap();
            file.write_all_at(&vec![0; length], 0).await.unwrap();
            assert_eq!(adapter::load_core(path, Mode::Recover).await.err().unwrap().kind(), io::ErrorKind::InvalidData);
        }
        let mut bytes = u64::MAX.to_le_bytes().to_vec();
        bytes.extend(9u64.to_le_bytes());
        let file = fs::File::create(path).await.unwrap();
        file.write_all_at(&bytes, 0).await.unwrap();
        assert_eq!(adapter::load_core(path, Mode::Recover).await.err().unwrap().kind(), io::ErrorKind::InvalidData);
        assert_eq!(adapter::load_core(path, Mode::Fresh).await.err().unwrap().kind(), io::ErrorKind::AlreadyExists);
        // Boundary fixture, not a claim of billions of completed operations.
        bytes[..8].copy_from_slice(&(u64::MAX - 1).to_le_bytes());
        let file = fs::File::create(path).await.unwrap();
        file.write_all_at(&bytes, 0).await.unwrap();
        let mut core = adapter::load_core(path, Mode::Recover).await.unwrap();
        assert_eq!((core.value(), core.next_id(), core.pending()), (9, u64::MAX, None));
        assert_eq!(core.begin(put(11)), Begin::Exhausted);
    })).await.unwrap();
}
