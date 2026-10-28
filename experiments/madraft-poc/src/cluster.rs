//! DRAFT actual MadSim cluster. Protocol decisions belong exclusively to Node::handle_event.
use async_channel::{Receiver, Sender};
use madsim::{
    net::{Endpoint, NetSim},
    runtime::{Handle, NodeHandle, Runtime},
    time::{sleep, Duration},
};
use raft_verified::*;
use std::{
    collections::BTreeMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Saved {
    pub hard: DurableState,
    pub incarnation: u64,
    pub applied: Vec<Entry>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Record {
    Step(u64, Event, Result<Vec<Effect>, InputError>),
    Durable(u64, OperationId, DurableState),
    Applied(u64, u64, u64, Entry),
    Crash(u64, CrashPoint),
    Restart(u64),
    Partition(u64, bool),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrashPoint {
    BeforeWrite,
    AfterWrite,
    AfterApply,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct History {
    pub records: Vec<Record>,
    pub nodes: BTreeMap<u64, Observation>,
    pub disks: BTreeMap<u64, Saved>,
    pub elections: Vec<(u64, u64, Vec<Entry>)>,
    pub commits: Vec<(u64, Vec<Entry>)>,
    pub applications: Vec<(u64, u64, Entry)>,
    crash: Option<(u64, CrashPoint)>,
    duplicate: bool,
}
type Shared = Arc<Mutex<History>>;
fn address(id: u64) -> SocketAddr {
    SocketAddr::from(([10, 0, 0, id as u8], 9100))
}

// The codec transports actual commands; no-op substitution cannot pass application checks.
pub fn encode(m: &Message) -> Vec<u8> {
    let mut v = Vec::new();
    fn word(v: &mut Vec<u8>, x: u64) {
        v.extend_from_slice(&x.to_le_bytes());
    }
    for x in [m.from, m.to, m.term, m.rpc.incarnation, m.rpc.sequence] {
        word(&mut v, x);
    }
    match &m.body {
        MessageBody::RequestVote {
            last_index,
            last_term,
        } => {
            for x in [0, *last_index, *last_term] {
                word(&mut v, x);
            }
        }
        MessageBody::VoteResponse { granted } => {
            word(&mut v, 1);
            word(&mut v, *granted as u64);
        }
        MessageBody::AppendEntries {
            previous_index,
            previous_term,
            entries,
            leader_commit,
        } => {
            for x in [
                2,
                *previous_index,
                *previous_term,
                *leader_commit,
                entries.len() as u64,
            ] {
                word(&mut v, x);
            }
            for e in entries {
                word(&mut v, e.term);
                match &e.command {
                    Command::Noop => word(&mut v, 0),
                    Command::Client { request, bytes } => {
                        for x in [1, request.client, request.sequence, bytes.len() as u64] {
                            word(&mut v, x);
                        }
                        v.extend_from_slice(bytes);
                    }
                }
            }
        }
        MessageBody::AppendResponse {
            accepted,
            matched_index,
        } => {
            for x in [3, *accepted as u64, *matched_index] {
                word(&mut v, x);
            }
        }
    }
    v
}
pub fn decode(mut bytes: &[u8]) -> Option<Message> {
    fn word(b: &mut &[u8]) -> Option<u64> {
        if b.len() < 8 {
            return None;
        }
        let n = u64::from_le_bytes(b[..8].try_into().ok()?);
        *b = &b[8..];
        Some(n)
    }
    fn flag(b: &mut &[u8]) -> Option<bool> {
        match word(b)? {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }
    }
    let from = word(&mut bytes)?;
    let to = word(&mut bytes)?;
    let term = word(&mut bytes)?;
    let rpc = OperationId {
        incarnation: word(&mut bytes)?,
        sequence: word(&mut bytes)?,
    };
    let body = match word(&mut bytes)? {
        0 => MessageBody::RequestVote {
            last_index: word(&mut bytes)?,
            last_term: word(&mut bytes)?,
        },
        1 => MessageBody::VoteResponse {
            granted: flag(&mut bytes)?,
        },
        2 => {
            let previous_index = word(&mut bytes)?;
            let previous_term = word(&mut bytes)?;
            let leader_commit = word(&mut bytes)?;
            let count = word(&mut bytes)?;
            if count > bytes.len() as u64 / 16 {
                return None;
            }
            let mut entries = Vec::new();
            for _ in 0..count {
                let term = word(&mut bytes)?;
                let command = match word(&mut bytes)? {
                    0 => Command::Noop,
                    1 => {
                        let request = RequestId {
                            client: word(&mut bytes)?,
                            sequence: word(&mut bytes)?,
                        };
                        let len = usize::try_from(word(&mut bytes)?).ok()?;
                        let data = bytes.get(..len)?.to_vec();
                        bytes = &bytes[len..];
                        Command::Client {
                            request,
                            bytes: data,
                        }
                    }
                    _ => return None,
                };
                entries.push(Entry { term, command });
            }
            MessageBody::AppendEntries {
                previous_index,
                previous_term,
                entries,
                leader_commit,
            }
        }
        3 => MessageBody::AppendResponse {
            accepted: flag(&mut bytes)?,
            matched_index: word(&mut bytes)?,
        },
        _ => return None,
    };
    if bytes.is_empty() {
        Some(Message {
            from,
            to,
            term,
            rpc,
            body,
        })
    } else {
        None
    }
}
fn assert_safety(h: &History) {
    for (a, t, _) in &h.elections {
        for (b, u, _) in &h.elections {
            if t == u {
                assert_eq!(a, b, "Election Safety");
            }
        }
    }
    for a in h.nodes.values() {
        for b in h.nodes.values() {
            for i in 0..a.hard.log.len().min(b.hard.log.len()) {
                if a.hard.log[i].term == b.hard.log[i].term {
                    assert_eq!(a.hard.log[..=i], b.hard.log[..=i], "Log Matching");
                }
            }
        }
    }
    for (term, prefix) in &h.commits {
        for (_, later, log) in &h.elections {
            if later > term {
                assert!(log.starts_with(prefix), "Leader Completeness");
            }
        }
    }
    for (_, i, a) in &h.applications {
        for (_, j, b) in &h.applications {
            if i == j {
                assert_eq!(a, b, "State Machine Safety");
            }
        }
    }
}
fn crash_at(id: u64, point: CrashPoint, shared: &Shared) -> bool {
    let trigger = {
        let mut h = shared.lock().unwrap();
        if h.crash == Some((id, point)) {
            h.crash = None;
            h.records.push(Record::Crash(id, point));
            h.nodes.remove(&id);
            true
        } else {
            false
        }
    };
    if trigger {
        Handle::current().kill(format!("node-{id}"));
    }
    trigger
}
fn effect(id: u64, e: Effect, ep: Endpoint, tx: Sender<Event>, shared: Shared) {
    madsim::task::spawn(async move {
        match e {
            Effect::Send { operation, message } => {
                let bytes = encode(&message);
                let duplicate = shared.lock().unwrap().duplicate;
                let outcome = if ep.send_to(address(message.to), 0, &bytes).await.is_ok() {
                    CompletionOutcome::Success
                } else {
                    CompletionOutcome::NotPerformed
                };
                if duplicate {
                    sleep(Duration::from_millis(2)).await;
                    let _ = ep.send_to(address(message.to), 0, &bytes).await;
                }
                let _ = tx
                    .send(Event::Complete {
                        operation,
                        kind: CompletionKind::Send,
                        outcome,
                    })
                    .await;
            }
            Effect::Persist { operation, state } => {
                sleep(Duration::from_millis(3)).await;
                if crash_at(id, CrashPoint::BeforeWrite, &shared) {
                    return;
                }
                {
                    let mut h = shared.lock().unwrap();
                    let disk = h.disks.get_mut(&id).unwrap();
                    disk.hard = state.clone();
                    disk.incarnation = operation.incarnation;
                    h.records.push(Record::Durable(id, operation, state));
                }
                if crash_at(id, CrashPoint::AfterWrite, &shared) {
                    return;
                }
                sleep(Duration::from_millis(3)).await;
                let _ = tx
                    .send(Event::Complete {
                        operation,
                        kind: CompletionKind::Persist,
                        outcome: CompletionOutcome::Success,
                    })
                    .await;
            }
            Effect::Deliver {
                operation,
                index,
                entry,
            } => {
                sleep(Duration::from_millis(1)).await;
                {
                    let mut h = shared.lock().unwrap();
                    let d = h.disks.get_mut(&id).unwrap();
                    if index as usize == d.applied.len() + 1 {
                        d.applied.push(entry.clone());
                    } else {
                        assert_eq!(d.applied.get(index as usize - 1), Some(&entry));
                    }
                    h.applications.push((id, index, entry.clone()));
                    h.records
                        .push(Record::Applied(id, operation.incarnation, index, entry));
                    assert_safety(&h);
                }
                if crash_at(id, CrashPoint::AfterApply, &shared) {
                    return;
                }
                let _ = tx
                    .send(Event::Complete {
                        operation,
                        kind: CompletionKind::Apply,
                        outcome: CompletionOutcome::Success,
                    })
                    .await;
            }
            Effect::ResetElectionTimer {
                incarnation,
                generation,
            } => {
                sleep(Duration::from_millis(160 + id * 37)).await;
                let _ = tx
                    .send(Event::ElectionTimeout {
                        incarnation,
                        generation,
                    })
                    .await;
            }
            Effect::ResetHeartbeatTimer {
                incarnation,
                generation,
            } => {
                sleep(Duration::from_millis(35)).await;
                let _ = tx
                    .send(Event::HeartbeatTimeout {
                        incarnation,
                        generation,
                    })
                    .await;
            }
            Effect::ProposalAccepted { .. }
            | Effect::CancelTimer { .. }
            | Effect::Reject { .. } => {}
        }
    });
}
async fn actor(id: u64, members: Vec<u64>, rx: Receiver<Event>, tx: Sender<Event>, shared: Shared) {
    // This is a fresh volatile instance, including after restart; no old Node survives a crash.
    let mut core = Node::new(id, members).unwrap();
    let ep = Endpoint::bind(address(id)).await.unwrap();
    let (net, events) = (ep.clone(), tx.clone());
    madsim::task::spawn(async move {
        loop {
            let mut buf = vec![0; 1 << 20];
            let (len, peer) = net.recv_from(0, &mut buf).await.unwrap();
            let message = decode(&buf[..len]).expect("invalid transport encoding");
            assert_eq!(peer, address(message.from));
            if events.send(Event::Receive(message)).await.is_err() {
                return;
            }
        }
    });
    shared
        .lock()
        .unwrap()
        .nodes
        .insert(id, core.observe().clone());
    while let Ok(event) = rx.recv().await {
        let before = core.observe().clone();
        let result = core.handle_event(event.clone());
        let after = core.observe().clone();
        {
            let mut h = shared.lock().unwrap();
            if before.role == Role::Leader
                && after.role == Role::Leader
                && before.hard.term == after.hard.term
                && !before.recovery_required
                && !after.recovery_required
            {
                assert!(
                    after.hard.log.starts_with(&before.hard.log),
                    "Leader Append-Only while leading the same term"
                );
            }
            if after.commit > before.commit {
                h.commits.push((
                    after.hard.term,
                    after.hard.log[..after.commit as usize].to_vec(),
                ));
            }
            if after.role == Role::Leader
                && (before.role != Role::Leader || before.hard.term != after.hard.term)
            {
                h.elections
                    .push((id, after.hard.term, after.hard.log.clone()));
            }
            h.nodes.insert(id, after);
            h.records.push(Record::Step(id, event, result.clone()));
            assert_safety(&h);
        }
        if let Ok(out) = result {
            for e in out {
                effect(id, e, ep.clone(), tx.clone(), shared.clone());
            }
        }
    }
}
pub struct Cluster {
    shared: Shared,
    nodes: BTreeMap<u64, (NodeHandle, Sender<Event>)>,
    members: Vec<u64>,
}
impl Cluster {
    pub async fn new(count: u64) -> Self {
        assert!((1..=5).contains(&count)); // test harness address range only; core membership is parameterized
        let mut c = Self {
            shared: Arc::new(Mutex::new(History::default())),
            nodes: BTreeMap::new(),
            members: (1..=count).collect(),
        };
        for id in c.members.clone() {
            let node = Handle::current()
                .create_node()
                .name(format!("node-{id}"))
                .ip(address(id).ip())
                .build();
            c.shared.lock().unwrap().disks.insert(
                id,
                Saved {
                    hard: Node::new(id, c.members.clone())
                        .unwrap()
                        .observe()
                        .hard
                        .clone(),
                    incarnation: 0,
                    applied: vec![],
                },
            );
            c.start(id, node, false).await;
        }
        c
    }
    async fn start(&mut self, id: u64, node: NodeHandle, recover: bool) {
        let (tx, rx) = async_channel::unbounded();
        let members = self.members.clone();
        let h = self.shared.clone();
        let sender = tx.clone();
        node.spawn(actor(id, members, rx, sender, h));
        self.nodes.insert(id, (node, tx.clone()));
        if recover {
            let d = self.shared.lock().unwrap().disks[&id].clone();
            tx.send(Event::Recover {
                incarnation: d.incarnation.checked_add(1).expect("incarnation exhausted"),
                durable: d.hard,
                application_cursor: d.applied.len() as u64,
            })
            .await
            .unwrap();
        } else {
            madsim::task::spawn(async move {
                sleep(Duration::from_millis(160 + id * 37)).await;
                let _ = tx
                    .send(Event::ElectionTimeout {
                        incarnation: 0,
                        generation: 1,
                    })
                    .await;
            });
        }
        sleep(Duration::from_millis(1)).await;
    }
    pub fn history(&self) -> History {
        self.shared.lock().unwrap().clone()
    }
    pub fn leader(&self, exclude: Option<u64>) -> u64 {
        self.shared
            .lock()
            .unwrap()
            .nodes
            .iter()
            .filter(|(id, n)| {
                Some(**id) != exclude && n.role == Role::Leader && !n.recovery_required
            })
            .max_by_key(|(_, n)| n.hard.term)
            .map(|(id, _)| *id)
            .expect("no elected leader")
    }
    pub async fn event(&self, id: u64, event: Event) {
        self.nodes[&id].1.send(event).await.unwrap();
    }
    pub async fn propose(&self, id: u64, sequence: u64, bytes: Vec<u8>) {
        self.event(
            id,
            Event::Propose {
                request: RequestId {
                    client: 7,
                    sequence,
                },
                bytes,
            },
        )
        .await;
    }
    pub fn partition(&self, id: u64, yes: bool) {
        let node = self.nodes[&id].0.id();
        if yes {
            NetSim::current().clog_node(node)
        } else {
            NetSim::current().unclog_node(node)
        }
        self.shared
            .lock()
            .unwrap()
            .records
            .push(Record::Partition(id, yes));
    }
    pub fn crash_next(&self, id: u64, point: CrashPoint) {
        self.shared.lock().unwrap().crash = Some((id, point));
    }
    pub async fn restart(&mut self, id: u64) {
        let node = self.nodes[&id].0.clone();
        Handle::current().restart(node.id());
        let node = Handle::current().get_node(node.id()).unwrap();
        self.shared
            .lock()
            .unwrap()
            .records
            .push(Record::Restart(id));
        self.start(id, node, true).await;
    }
    pub fn duplicate(&self) {
        self.shared.lock().unwrap().duplicate = true;
    }
    pub async fn settle(&self, ms: u64) {
        sleep(Duration::from_millis(ms)).await;
    }
}
#[derive(Clone, Copy, Debug)]
pub enum Scenario {
    Nominal(u64),
    Partition,
    Restart(CrashPoint),
    Duplicate,
    Competing,
}
pub fn run(scenario: Scenario, seed: u64) -> History {
    Runtime::with_seed_and_config(seed, Default::default()).block_on(async {
        let count = match scenario {
            Scenario::Nominal(n) => n,
            _ => 3,
        };
        let mut c = Cluster::new(count).await;
        if matches!(scenario, Scenario::Duplicate) {
            c.duplicate();
            NetSim::current().update_config(|n| {
                n.packet_loss_rate = 0.1;
                n.send_latency = Duration::from_millis(2)..Duration::from_millis(9);
            });
        }
        if matches!(scenario, Scenario::Competing) {
            for id in 1..=count {
                c.event(
                    id,
                    Event::ElectionTimeout {
                        incarnation: 0,
                        generation: 1,
                    },
                )
                .await;
            }
        }
        c.settle(1100).await;
        let leader = c.leader(None);
        for i in 1..=3 {
            c.propose(leader, i, vec![i as u8, 42]).await;
            c.settle(180).await;
        }
        match scenario {
            Scenario::Partition => {
                c.partition(leader, true);
                c.propose(leader, 40, vec![40]).await;
                c.settle(650).await;
                let next = c.leader(Some(leader));
                c.propose(next, 50, vec![50]).await;
                c.settle(250).await;
                c.partition(leader, false);
                c.settle(900).await;
            }
            Scenario::Restart(point) => {
                c.crash_next(leader, point);
                c.propose(leader, 60, vec![60, 61]).await;
                c.settle(600).await;
                assert!(c
                    .history()
                    .records
                    .iter()
                    .any(|r| matches!(r,Record::Crash(id,p) if *id==leader && *p==point)));
                c.restart(leader).await;
                c.settle(900).await;
                let next = c.leader(None);
                c.propose(next, 70, vec![70]).await;
                c.settle(700).await;
            }
            _ => {}
        }
        c.settle(500).await;
        let h = c.history();
        assert_safety(&h);
        h
    })
}
