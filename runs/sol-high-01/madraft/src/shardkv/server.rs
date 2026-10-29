use super::{msg::*, key2shard};
use crate::{kvraft::server::{Server, State}, shard_ctrler::{client::Clerk as CtrlerClerk, msg::Config, N_SHARDS}};
use futures::future::join_all;
use madsim::{task, time::*};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, net::SocketAddr, sync::Arc};

pub struct ShardKvServer { inner: Arc<Server<ShardKv>> }
impl ShardKvServer {
    pub async fn new(ctrl_ck: CtrlerClerk, servers: Vec<SocketAddr>, gid: u64, me: usize, max_raft_state: Option<usize>) -> Arc<Self> {
        let initial = ShardKv { gid, ..ShardKv::default() };
        let inner = Server::new_with_state(servers, me, max_raft_state, initial).await;
        // Reuse this host's concrete endpoint. A wildcard ephemeral bind in
        // MadSim can choose the same port as the concrete Raft listener, which
        // would route controller replies to the wrong endpoint.
        ctrl_ck.use_endpoint(inner.rf.endpoint.clone());
        let this = Arc::new(Self { inner });
        let copy = this.clone();
        this.inner.rf.endpoint.add_rpc_handler(move |f: Fetch| {
            let copy = copy.clone();
            async move {
                // An export exists only after a committed configuration froze it.
                copy.inner.service.lock().unwrap().state.outgoing.get(&(f.num, f.shard)).cloned()
            }
        });
        let copy = this.clone();
        this.inner.rf.endpoint.add_rpc_handler(move |f: Delete| {
            let copy = copy.clone();
            async move {
                if f.shard >= N_SHARDS { return false; }
                {
                    let service = copy.inner.service.lock().unwrap();
                    if service.state.config.num < f.num { return false; }
                    if !service.state.outgoing.contains_key(&(f.num, f.shard)) { return true; }
                }
                matches!(copy.inner.submit(Op::Forget { num: f.num, shard: f.shard }).await, Ok(Reply::Ok))
            }
        });
        let copy = this.clone();
        task::spawn(async move {
            loop {
                sleep(Duration::from_millis(60)).await;
                if !copy.inner.is_leader() { continue; }
                let next = {
                    let service = copy.inner.service.lock().unwrap();
                    if !service.state.incoming.is_empty() { None }
                    else { service.state.config.num.checked_add(1) }
                };
                if let Some(next) = next {
                    let config = ctrl_ck.query_at(next).await;
                    if config.num == next { let _ = copy.inner.submit(Op::Configuration { config }).await; }
                }
            }
        });
        let copy = this.clone();
        task::spawn(async move {
            loop {
                sleep(Duration::from_millis(35)).await;
                if !copy.inner.is_leader() { continue; }
                let (incoming, gc) = {
                    let service = copy.inner.service.lock().unwrap();
                    let state = &service.state;
                    (state.incoming.iter().map(|(&shard, peers)| (state.config.num, shard, peers.clone())).collect::<Vec<_>>(), state.gc.clone())
                };
                let mut work = vec![];
                for (num, shard, peers) in incoming {
                    let copy = copy.clone();
                    work.push(task::spawn(async move {
                        for peer in peers {
                            if let Ok(Some(data)) = copy.inner.rf.endpoint.call_timeout(peer, Fetch { num, shard }, Duration::from_millis(180)).await {
                                let _ = copy.inner.submit(Op::Import { num, shard, data }).await;
                                break;
                            }
                        }
                    }));
                }
                for ((num, shard), peers) in gc {
                    let copy = copy.clone();
                    work.push(task::spawn(async move {
                        for peer in peers {
                            if let Ok(true) = copy.inner.rf.endpoint.call_timeout(peer, Delete { num, shard }, Duration::from_millis(350)).await {
                                let _ = copy.inner.submit(Op::FinishGc { num, shard }).await;
                                break;
                            }
                        }
                    }));
                }
                join_all(work).await;
            }
        });
        this
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ShardKv {
    gid: u64,
    config: Config,
    shards: BTreeMap<usize, ShardData>,
    incoming: BTreeMap<usize, Vec<SocketAddr>>,
    outgoing: BTreeMap<(u64, usize), ShardData>,
    gc: BTreeMap<(u64, usize), Vec<SocketAddr>>,
}
impl State for ShardKv {
    type Command = Op;
    type Output = Reply;
    fn deduplicate() -> bool { false }
    fn apply(&mut self, cmd: Op) -> Reply { self.apply_request(0, 0, cmd) }
    fn apply_request(&mut self, client: u128, sequence: u64, cmd: Op) -> Reply {
        match cmd {
            Op::Configuration { config } => {
                if config.num != self.config.num.saturating_add(1) || !self.incoming.is_empty() { return Reply::WrongGroup; }
                for shard in 0..N_SHARDS {
                    let (old, new) = (self.config.shards[shard], config.shards[shard]);
                    if old == self.gid && new != self.gid {
                        if let Some(data) = self.shards.remove(&shard) {
                            if new != 0 { self.outgoing.insert((config.num, shard), data); }
                        }
                    } else if old != self.gid && new == self.gid {
                        if old == 0 { self.shards.insert(shard, ShardData::default()); }
                        else if let Some(peers) = self.config.groups.get(&old) { self.incoming.insert(shard, peers.clone()); }
                    }
                }
                self.config = config;
                Reply::Ok
            }
            Op::Import { num, shard, data } => {
                if self.config.num == num && shard < N_SHARDS && self.config.shards[shard] == self.gid {
                    if let Some(peers) = self.incoming.remove(&shard) {
                        self.shards.insert(shard, data);
                        self.gc.insert((num, shard), peers);
                    }
                }
                Reply::Ok
            }
            Op::Forget { num, shard } => { self.outgoing.remove(&(num, shard)); Reply::Ok }
            Op::FinishGc { num, shard } => { self.gc.remove(&(num, shard)); Reply::Ok }
            operation => {
                let key = match &operation { Op::Get { key } | Op::Put { key, .. } | Op::Append { key, .. } => key, _ => unreachable!() };
                let shard = key2shard(key);
                if self.config.shards[shard] != self.gid || self.incoming.contains_key(&shard) { return Reply::WrongGroup; }
                let data = match self.shards.get_mut(&shard) { Some(s) => s, None => return Reply::WrongGroup };
                if let Some((seq, reply)) = data.sessions.get(&client) {
                    if *seq == sequence { return reply.clone(); }
                    if *seq > sequence { return Reply::WrongGroup; }
                }
                let result = match operation {
                    Op::Get { key } => Reply::Get { value: data.values.get(&key).cloned() },
                    Op::Put { key, value } => { data.values.insert(key, value); Reply::Ok }
                    Op::Append { key, value } => { data.values.entry(key).or_default().push_str(&value); Reply::Ok }
                    _ => unreachable!(),
                };
                data.sessions.insert(client, (sequence, result.clone()));
                result
            }
        }
    }
}
