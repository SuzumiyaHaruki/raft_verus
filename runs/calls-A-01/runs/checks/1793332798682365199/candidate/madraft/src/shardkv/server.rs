use super::msg::*;
use crate::kvraft::{server::{Server, State}, msg::ClientRequest};
use crate::shard_ctrler::{client::Clerk as CtrlerClerk, msg::Config, N_SHARDS};
use madsim::time::*;
use serde::{Deserialize, Serialize};
use std::{collections::{BTreeMap, BTreeSet}, net::SocketAddr, sync::Arc};

pub struct ShardKvServer { inner: Arc<Server<ShardKv>> }
impl ShardKvServer {
    pub async fn new(ctrl_ck: CtrlerClerk, servers: Vec<SocketAddr>, gid: u64, me: usize, max_raft_state: Option<usize>) -> Arc<Self> {
        let inner = Server::new(servers, me, max_raft_state).await;
        let this = Arc::new(Self { inner });
        let h = this.clone();
        this.inner.endpoint().add_rpc_handler(move |p: Pull| {
            let h = h.clone();
            async move { h.inner.inspect(|s| s.outgoing.get(&(p.config, p.shard)).cloned()) }
        });
        let h = this.clone();
        this.inner.endpoint().add_rpc_handler(move |p: Collect| {
            let h = h.clone();
            async move {
                if !h.inner.inspect(|s| s.config.num >= p.config) { return false; }
                if h.inner.inspect(|s| !s.outgoing.contains_key(&(p.config, p.shard))) { return true; }
                h.inner.apply(ClientRequest { client: u128::MAX - p.shard as u128, sequence: p.config,
                    command: Op::Delete { config: p.config, shard: p.shard } }).await.is_ok()
            }
        });
        let h = this.clone();
        madsim::task::spawn(async move {
            let client: u128 = madsim::rand::random();
            let mut sequence = 0u64;
            loop {
                sleep(Duration::from_millis(60)).await;
                if !h.inner.is_leader() { continue; }
                let (current, incoming, previous, gc, initialized) = h.inner.inspect(|s|
                    (s.config.clone(), s.incoming.clone(), s.previous.clone(), s.gc.clone(), s.gid != 0));
                let mut commands = vec![];
                if !initialized { commands.push(Op::Init { gid }); }
                else {
                    for shard in incoming.iter().copied() {
                        let old = previous.shards[shard];
                        if let Some(peers) = previous.groups.get(&old) {
                            for addr in peers {
                                if let Ok(Some(data)) = h.inner.endpoint().call_timeout(*addr, Pull { config: current.num, shard }, Duration::from_millis(200)).await {
                                    commands.push(Op::Install { config: current.num, shard, data }); break;
                                }
                            }
                        }
                    }
                    for ((config, shard), peers) in gc {
                        for addr in peers {
                            if let Ok(true) = h.inner.endpoint().call_timeout(addr, Collect { config, shard }, Duration::from_millis(250)).await {
                                commands.push(Op::GcDone { config, shard }); break;
                            }
                        }
                    }
                    if incoming.is_empty() {
                        if let Some(next) = current.num.checked_add(1) {
                            let config = ctrl_ck.query_at(next).await;
                            if config.num == next { commands.push(Op::Configure { config }); }
                        }
                    }
                }
                for command in commands {
                    let Some(next) = sequence.checked_add(1) else { return; };
                    sequence = next;
                    let _ = h.inner.apply(ClientRequest { client, sequence, command }).await;
                }
            }
        });
        this
    }
}
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ShardKv {
    gid: u64, config: Config, previous: Config,
    shards: BTreeMap<usize, ShardData>, incoming: BTreeSet<usize>,
    outgoing: BTreeMap<(u64, usize), ShardData>,
    gc: BTreeMap<(u64, usize), Vec<SocketAddr>>,
}
impl State for ShardKv {
    type Command = Op;
    type Output = Reply;
    const DURABLE_REPLIES: bool = false;
    fn retry_output(output: &Reply) -> bool { matches!(output, Reply::WrongGroup) }
    fn apply(&mut self, cmd: Op) -> Reply {
        match cmd {
            Op::Init { gid } => { if self.gid == 0 { self.gid = gid; } }
            Op::Configure { config } => {
                if self.config.num.checked_add(1) == Some(config.num) && self.incoming.is_empty() {
                    for shard in 0..N_SHARDS {
                        if self.config.shards[shard] == self.gid && config.shards[shard] != self.gid {
                            if let Some(data) = self.shards.remove(&shard) {
                                if config.shards[shard] != 0 { self.outgoing.insert((config.num, shard), data); }
                            }
                        }
                        if config.shards[shard] == self.gid && self.config.shards[shard] != self.gid {
                            if self.config.shards[shard] == 0 { self.shards.insert(shard, ShardData::default()); }
                            else { self.incoming.insert(shard); }
                        }
                    }
                    self.previous = self.config.clone(); self.config = config;
                }
            }
            Op::Install { config, shard, data } => {
                if self.config.num == config && self.incoming.remove(&shard) {
                    self.shards.insert(shard, data);
                    if let Some(peers) = self.previous.groups.get(&self.previous.shards[shard]) {
                        self.gc.insert((config, shard), peers.clone());
                    }
                }
            }
            Op::Delete { config, shard } => { self.outgoing.remove(&(config, shard)); }
            Op::GcDone { config, shard } => { self.gc.remove(&(config, shard)); }
            Op::Client { client, sequence, config, operation } => {
                let key = match operation.as_ref() {
                    Op::Get { key } | Op::Put { key, .. } | Op::Append { key, .. } => key,
                    _ => return Reply::WrongGroup,
                };
                let shard = super::key2shard(key);
                if self.config.num != config || self.config.shards[shard] != self.gid || self.incoming.contains(&shard) { return Reply::WrongGroup; }
                let Some(data) = self.shards.get_mut(&shard) else { return Reply::WrongGroup; };
                if let Some((last, response)) = data.replies.get(&client) {
                    if *last >= sequence { return response.clone(); }
                }
                let response = match *operation {
                    Op::Get { key } => Reply::Get { value: data.values.get(&key).cloned() },
                    Op::Put { key, value } => { data.values.insert(key, value); Reply::Ok },
                    Op::Append { key, value } => { data.values.entry(key).or_default().push_str(&value); Reply::Ok },
                    _ => unreachable!(),
                };
                data.replies.insert(client, (sequence, response.clone()));
                return response;
            }
            _ => return Reply::WrongGroup,
        }
        Reply::Ok
    }
}
