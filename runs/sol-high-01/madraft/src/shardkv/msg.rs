use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use crate::shard_ctrler::msg::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Op {
    Get { key: String },
    Put { key: String, value: String },
    Append { key: String, value: String },
    Configuration { config: Config },
    Import { num: u64, shard: usize, data: ShardData },
    Forget { num: u64, shard: usize },
    FinishGc { num: u64, shard: usize },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Reply { Get { value: Option<String> }, Ok, WrongGroup }
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ShardData {
    pub values: BTreeMap<String, String>,
    pub sessions: BTreeMap<u128, (u64, Reply)>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fetch { pub num: u64, pub shard: usize }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delete { pub num: u64, pub shard: usize }
impl madsim::net::rpc::Request for Op {
    type Response = Result<Reply, crate::kvraft::msg::Error>;
    const ID: u64 = 4;
}
impl madsim::net::rpc::Request for Fetch {
    type Response = Option<ShardData>;
    const ID: u64 = 7;
}
impl madsim::net::rpc::Request for Delete {
    type Response = bool;
    const ID: u64 = 8;
}
