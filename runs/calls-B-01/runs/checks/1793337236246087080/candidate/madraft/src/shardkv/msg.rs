use serde::{Deserialize, Serialize};
use crate::shard_ctrler::msg::Config;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action { Get { key: String }, Put { key: String, value: String }, Append { key: String, value: String } }
impl Action { pub fn key(&self) -> &str { match self { Self::Get { key } | Self::Put { key, .. } | Self::Append { key, .. } => key } } }
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ShardData { pub values: HashMap<String, String>, pub sessions: HashMap<u64, (u64, Reply)> }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Op {
    Get { key: String }, Put { key: String, value: String }, Append { key: String, value: String },
    Request { client: u64, sequence: u64, action: Action },
    Configure { gid: u64, config: Config },
    Import { epoch: u64, shard: usize, data: ShardData },
    Delete { epoch: u64, shard: usize }, Acked { epoch: u64, shard: usize },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Reply { Get { value: Option<String> }, Ok, WrongGroup }
impl madsim::net::rpc::Request for Op {
    type Response = Result<Reply, crate::kvraft::msg::Error>;
    const ID: u64 = 4;
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Fetch { pub epoch: u64, pub shard: usize }
impl madsim::net::rpc::Request for Fetch { type Response = Option<ShardData>; const ID: u64 = 20001; }
#[derive(Clone, Serialize, Deserialize)]
pub struct Ack { pub epoch: u64, pub shard: usize }
impl madsim::net::rpc::Request for Ack { type Response = bool; const ID: u64 = 20002; }
