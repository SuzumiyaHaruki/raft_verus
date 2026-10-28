//! DRAFT thin MadSim adapter, adapted conceptually from MadRaft's node/network controls.
//! No election/vote decision is made here. The source of every injected request is the test.
#[cfg(not(madsim))]
compile_error!("Use the actual MadSim runtime with --cfg madsim");
use madsim::{
    net::{Endpoint, NetSim},
    runtime::{Handle, Runtime},
    time::{sleep, Duration},
};
use raft_verified::{madraft::VoteAction, *};
use std::{
    collections::BTreeMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scenario {
    Nominal,
    Barrier,
    Edges,
    BoundaryLog,
    Faults,
    Crash,
    Failed,
    Unknown,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Record {
    Receive(Message),
    Decision(String, Observation),
    Persist(OperationId, DurableState, bool),
    Durable(OperationId, DurableState),
    Complete(
        OperationId,
        CompletionOutcome,
        Result<Option<Message>, InputError>,
    ),
    Send(Message),
    Delivered(Message),
    Checkpoint,
    Loss(bool),
    Duplicate,
    Crash,
    Timer(String),
    BoundaryFixture,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Trace {
    pub records: Vec<Record>,
    pub disk: Option<DurableState>,
    pub final_state: Observation,
}
type Records = Arc<Mutex<Vec<Record>>>;
type Disk = Arc<Mutex<BTreeMap<u64, DurableState>>>;
fn log(records: &Records, r: Record) {
    records.lock().unwrap().push(r);
}
fn address(id: u64) -> SocketAddr {
    SocketAddr::from(([10, 0, 0, id as u8], 9000))
}

// Explicit codec only; unknown/truncated frames are errors, never votes.
fn encode(m: &Message) -> Vec<u8> {
    let (tag, a, b) = match m.body {
        MessageBody::RequestVote {
            last_index,
            last_term,
        } => (0, last_index, last_term),
        MessageBody::VoteResponse { granted } => (1, u64::from(granted), 0),
        _ => panic!("unsupported wire body"),
    };
    [
        m.from,
        m.to,
        m.term,
        m.rpc.incarnation,
        m.rpc.sequence,
        tag,
        a,
        b,
    ]
    .iter()
    .flat_map(|n| n.to_le_bytes())
    .collect()
}
fn decode(bytes: &[u8]) -> Option<Message> {
    if bytes.len() != 64 {
        return None;
    }
    let n: Vec<u64> = bytes
        .chunks_exact(8)
        .map(|x| u64::from_le_bytes(x.try_into().unwrap()))
        .collect();
    let body = match n[5] {
        0 => MessageBody::RequestVote {
            last_index: n[6],
            last_term: n[7],
        },
        1 if n[6] <= 1 && n[7] == 0 => MessageBody::VoteResponse { granted: n[6] == 1 },
        _ => return None,
    };
    Some(Message {
        from: n[0],
        to: n[1],
        term: n[2],
        rpc: OperationId {
            incarnation: n[3],
            sequence: n[4],
        },
        body,
    })
}
pub fn request(from: u64, term: u64, last_index: u64, last_term: u64, sequence: u64) -> Message {
    Message {
        from,
        to: 2,
        term,
        rpc: OperationId {
            incarnation: 0,
            sequence,
        },
        body: MessageBody::RequestVote {
            last_index,
            last_term,
        },
    }
}
async fn send(ep: &Endpoint, m: Message, records: &Records) {
    log(records, Record::Send(m.clone()));
    ep.send_to(address(m.to), 1, &encode(&m)).await.unwrap();
}
async fn serve(
    core: Arc<Mutex<Node>>,
    ep: Endpoint,
    m: Message,
    disk: Disk,
    records: Records,
    scenario: Scenario,
) {
    // At most two rounds: higher-term durability, then the retained RequestVote.
    // Each round is requested by core output; the host never computes grant eligibility.
    for _ in 0..2 {
        log(&records, Record::Receive(m.clone()));
        let action = {
            let mut n = core.lock().unwrap();
            let r = n.handle_request_vote(m.clone());
            log(
                &records,
                Record::Decision(format!("{r:?}"), n.observe().clone()),
            );
            r
        };
        match action {
            Err(_) => return,
            Ok(VoteAction::Reply(reply)) => {
                send(&ep, reply, &records).await;
                return;
            }
            Ok(VoteAction::Persist { operation, retry }) => {
                let image = core.lock().unwrap().observe().hard.clone();
                log(&records, Record::Persist(operation, image.clone(), retry));
                sleep(Duration::from_millis(10)).await;
                let outcome = match scenario {
                    Scenario::Failed => CompletionOutcome::NotPerformed,
                    Scenario::Unknown => CompletionOutcome::Unknown,
                    _ => CompletionOutcome::Success,
                };
                if outcome != CompletionOutcome::NotPerformed {
                    // Tested/trusted atomic whole-image durable store outside the killed node.
                    disk.lock().unwrap().insert(2, image.clone());
                    log(&records, Record::Durable(operation, image));
                }
                if scenario == Scenario::Crash {
                    log(&records, Record::Crash);
                    Handle::current().kill("follower");
                    return;
                }
                sleep(Duration::from_millis(
                    if matches!(scenario, Scenario::Barrier | Scenario::Faults) {
                        100
                    } else {
                        10
                    },
                ))
                .await;
                let result =
                    core.lock()
                        .unwrap()
                        .finish_vote(operation, CompletionKind::Persist, outcome);
                log(
                    &records,
                    Record::Complete(operation, outcome, result.clone()),
                );
                match result {
                    Ok(Some(reply)) => {
                        send(&ep, reply, &records).await;
                        return;
                    }
                    Ok(None) if retry => continue,
                    _ => return,
                }
            }
        }
    }
}

pub fn run(scenario: Scenario, seed: u64) -> Trace {
    let rt = Runtime::with_seed_and_config(seed, Default::default());
    rt.block_on(async {
        let h = Handle::current();
        let a = h
            .create_node()
            .name("injector-1")
            .ip(address(1).ip())
            .build();
        let b = h.create_node().name("follower").ip(address(2).ip()).build();
        let c = h
            .create_node()
            .name("injector-3")
            .ip(address(3).ip())
            .build();
        let mut n = Node::feasibility_follower(2, vec![1, 2, 3]).unwrap();
        let records: Records = Arc::new(Mutex::new(vec![]));
        if scenario == Scenario::BoundaryLog {
            let mut s = n.observe().clone();
            s.hard.term = 2;
            s.hard.log.push(Entry {
                term: 2,
                command: Command::Noop,
            });
            n = Node::from_observation(s);
            log(&records, Record::BoundaryFixture);
        }
        let core = Arc::new(Mutex::new(n));
        let disk: Disk = Arc::new(Mutex::new(BTreeMap::from([(
            2,
            core.lock().unwrap().observe().hard.clone(),
        )])));
        let (nc, ds, rs) = (core.clone(), disk.clone(), records.clone());
        let server = b.spawn(async move {
            let ep = Endpoint::bind(address(2)).await.unwrap();
            loop {
                let mut buf = [0; 65];
                let (len, peer) = ep.recv_from(0, &mut buf).await.unwrap();
                let m = decode(&buf[..len]).expect("invalid test wire input");
                assert_eq!(address(m.from), peer, "host identity binding");
                madsim::task::spawn(serve(
                    nc.clone(),
                    ep.clone(),
                    m,
                    ds.clone(),
                    rs.clone(),
                    scenario,
                ));
            }
        });
        // Bind injectors in their actual simulated node contexts.
        let ep1 = a
            .spawn(async { Endpoint::bind(address(1)).await.unwrap() })
            .await
            .unwrap();
        let ep3 = c
            .spawn(async { Endpoint::bind(address(3)).await.unwrap() })
            .await
            .unwrap();
        for (node, ep) in [(a.clone(), ep1.clone()), (c.clone(), ep3.clone())] {
            let rs = records.clone();
            node.spawn(async move {
                loop {
                    let mut buf = [0; 65];
                    let (len, peer) = ep.recv_from(1, &mut buf).await.unwrap();
                    let m = decode(&buf[..len]).unwrap();
                    assert_eq!(peer, address(2));
                    log(&rs, Record::Delivered(m));
                }
            });
        }
        sleep(Duration::from_millis(20)).await;
        let inputs = match scenario {
            Scenario::Edges => vec![
                request(1, 0, 0, 0, 1),
                request(1, 0, 0, 0, 2),
                request(3, 0, 0, 0, 3),
                request(3, 2, 0, 0, 4),
                request(1, 1, 0, 0, 5),
                request(1, 2, 0, 0, 6),
            ],
            Scenario::BoundaryLog => vec![
                request(1, 2, 99, 1, 1),
                request(1, 2, 0, 2, 2),
                request(1, 2, 1, 2, 3),
            ],
            _ => vec![request(1, 0, 0, 0, 1)],
        };
        if scenario == Scenario::Faults {
            NetSim::current().update_config(|x| x.packet_loss_rate = 1.0);
            log(&records, Record::Loss(true));
            let ep = ep1.clone();
            a.spawn(async move {
                ep.send_to(address(2), 0, &encode(&request(1, 0, 0, 0, 99)))
                    .await
                    .unwrap();
            })
            .await
            .unwrap();
            sleep(Duration::from_millis(70)).await;
            log(&records, Record::Checkpoint);
            NetSim::current().update_config(|x| {
                x.packet_loss_rate = 0.0;
                x.send_latency = Duration::from_millis(30)..Duration::from_millis(31);
            });
            log(&records, Record::Loss(false));
        }
        for m in inputs {
            let (node, ep) = if m.from == 1 {
                (a.clone(), ep1.clone())
            } else {
                (c.clone(), ep3.clone())
            };
            let dup = scenario == Scenario::Faults;
            let rs = records.clone();
            node.spawn(async move {
                ep.send_to(address(2), 0, &encode(&m)).await.unwrap();
                if dup {
                    log(&rs, Record::Duplicate);
                    ep.send_to(address(2), 0, &encode(&m)).await.unwrap();
                }
            })
            .await
            .unwrap();
            if scenario == Scenario::Barrier {
                sleep(Duration::from_millis(50)).await;
                log(&records, Record::Checkpoint);
            }
            sleep(Duration::from_millis(350)).await;
        }
        if scenario != Scenario::Crash {
            let nc = core.clone();
            let rs = records.clone();
            b.spawn(async move {
                sleep(Duration::from_millis(5)).await;
                let r = nc.lock().unwrap().step(Event::ElectionTimeout {
                    incarnation: 0,
                    generation: 1,
                });
                log(&rs, Record::Timer(format!("{r:?}")));
            })
            .await
            .unwrap();
        }
        server.abort();
        let final_state = core.lock().unwrap().observe().clone();
        let durable = disk.lock().unwrap().get(&2).cloned();
        let records = std::mem::take(&mut *records.lock().unwrap());
        Trace {
            records,
            disk: durable,
            final_state,
        }
    })
}
