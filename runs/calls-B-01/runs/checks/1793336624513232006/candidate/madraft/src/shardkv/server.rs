use super::msg::*;
use crate::{kvraft::server::{Server, State}, shard_ctrler::{client::Clerk as CtrlerClerk, msg::Config, N_SHARDS}};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc, collections::HashMap};
use madsim::{task, time::{sleep, Duration}};
pub struct ShardKvServer { inner: Arc<Server<ShardKv>> }
impl ShardKvServer {
    pub async fn new(mut ctrl_ck: CtrlerClerk, servers: Vec<SocketAddr>, gid: u64, me: usize, max_raft_state: Option<usize>) -> Arc<Self> {
        ctrl_ck.source_ip(servers[me].ip());
        let inner = Server::new(servers, me, max_raft_state).await;
        let this = Arc::new(Self { inner });
        let rf = this.inner.rf.clone();
        let donor = this.inner.clone();
        rf.endpoint.add_rpc_handler(move |request: Fetch| {
            let donor = donor.clone();
            async move { donor.inspect(|s| s.outgoing.get(&(request.epoch, request.shard)).cloned()) }
        });
        let donor = this.inner.clone();
        rf.endpoint.add_rpc_handler(move |request: Ack| {
            let donor = donor.clone();
            async move { matches!(donor.execute(Op::Delete { epoch: request.epoch, shard: request.shard }).await, Ok(Reply::Ok)) }
        });
        let worker = this.clone();
        task::spawn(async move {
            loop {
                sleep(Duration::from_millis(80)).await;
                if !worker.inner.is_leader() { continue; }
                let pending = worker.inner.inspect(|s| {
                    (0..N_SHARDS).filter(|&i| s.config.shards[i] == gid && !s.ready[i])
                        .map(|i| (s.config.num, i, s.previous.groups.get(&s.previous.shards[i]).cloned().unwrap_or_default())).collect::<Vec<_>>()
                });
                for (epoch, shard, peers) in pending {
                    for peer in peers {
                        if let Ok(Some(data)) = worker.inner.rf.endpoint.call_timeout(peer, Fetch { epoch, shard }, Duration::from_millis(100)).await {
                            let _ = worker.inner.execute(Op::Import { epoch, shard, data }).await;
                            break;
                        }
                    }
                }
                let acks = worker.inner.inspect(|s| s.acks.iter().map(|(&key, peers)| (key, peers.clone())).collect::<Vec<_>>());
                for ((epoch, shard), peers) in acks {
                    for peer in peers {
                        if let Ok(true) = worker.inner.rf.endpoint.call_timeout(peer, Ack { epoch, shard }, Duration::from_millis(100)).await {
                            let _ = worker.inner.execute(Op::Acked { epoch, shard }).await;
                            break;
                        }
                    }
                }
                let (num, ready) = worker.inner.inspect(|s| (s.config.num, s.complete()));
                if ready {
                    if let Some(next) = num.checked_add(1) {
                        let config = ctrl_ck.query_at(next).await;
                        if config.num == next { let _ = worker.inner.execute(Op::Configure { gid, config }).await; }
                    }
                }
            }
        });
        this
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ShardKv {
    gid: u64, config: Config, previous: Config,
    shards: Vec<ShardData>, ready: [bool; N_SHARDS],
    outgoing: HashMap<(u64, usize), ShardData>,
    acks: HashMap<(u64, usize), Vec<SocketAddr>>,
}
impl Default for ShardKv {
    fn default() -> Self { Self { gid: 0, config: Config::default(), previous: Config::default(), shards: vec![ShardData::default(); N_SHARDS],
        ready: [false; N_SHARDS], outgoing: HashMap::new(), acks: HashMap::new() } }
}
impl ShardKv {
    fn complete(&self) -> bool { self.gid == 0 || (0..N_SHARDS).all(|i| self.config.shards[i] != self.gid || self.ready[i]) }
    fn request(&mut self, client: u64, sequence: u64, action: Action) -> Reply {
        let shard = super::key2shard(action.key());
        if self.gid == 0 || self.config.shards[shard] != self.gid || !self.ready[shard] { return Reply::WrongGroup; }
        let data = &mut self.shards[shard];
        if let Some((seq, output)) = data.sessions.get(&client) { if sequence <= *seq { return output.clone(); } }
        let reply = match action {
            Action::Get { key } => Reply::Get { value: data.values.get(&key).cloned() },
            Action::Put { key, value } => { data.values.insert(key, value); Reply::Ok }
            Action::Append { key, value } => { data.values.entry(key).or_default().push_str(&value); Reply::Ok }
        };
        data.sessions.insert(client, (sequence, reply.clone())); reply
    }
}
impl State for ShardKv {
    type Command = Op; type Output = Reply;
    fn apply(&mut self, cmd: Op) -> Reply {
        match cmd {
            Op::Request { client, sequence, action } => self.request(client, sequence, action),
            Op::Get { key } => self.request(0, 0, Action::Get { key }),
            Op::Put { key, value } => self.request(0, 0, Action::Put { key, value }),
            Op::Append { key, value } => self.request(0, 0, Action::Append { key, value }),
            Op::Configure { gid, config } => {
                if !super::transfer::next_configuration(self.config.num, config.num, self.complete(), self.gid, gid) { return Reply::WrongGroup; }
                self.gid = gid;
                for shard in 0..N_SHARDS {
                    let old = self.config.shards[shard]; let new = config.shards[shard];
                    if old == gid && new != gid {
                        self.outgoing.insert((config.num, shard), std::mem::take(&mut self.shards[shard]));
                    }
                    self.ready[shard] = new == gid && (old == gid && self.ready[shard] || old == 0);
                    if new == gid && old == 0 { self.shards[shard] = ShardData::default(); }
                }
                self.previous = std::mem::replace(&mut self.config, config); Reply::Ok
            }
            Op::Import { epoch, shard, data } => {
                if shard < N_SHARDS && super::transfer::can_import(self.config.num, epoch, self.config.shards[shard] == self.gid, self.ready[shard]) {
                    self.shards[shard] = data; self.ready[shard] = true;
                    let peers = self.previous.groups.get(&self.previous.shards[shard]).cloned().unwrap_or_default();
                    self.acks.insert((epoch, shard), peers);
                }
                Reply::Ok
            }
            Op::Delete { epoch, shard } => { if epoch <= self.config.num { self.outgoing.remove(&(epoch, shard)); Reply::Ok } else { Reply::WrongGroup } }
            Op::Acked { epoch, shard } => { self.acks.remove(&(epoch, shard)); Reply::Ok }
        }
    }
}
