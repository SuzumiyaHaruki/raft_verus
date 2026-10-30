use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Op {
    Get { key: String },
    Put { key: String, value: String },
    Append { key: String, value: String },
    Client { client: u128, sequence: u64, config: u64, operation: Box<Op> },
    Init { gid: u64 },
    Configure { config: crate::shard_ctrler::msg::Config },
    Install { config: u64, shard: usize, data: ShardData },
    Delete { config: u64, shard: usize },
    GcDone { config: u64, shard: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Reply {
    Get { value: Option<String> },
    Ok,
    WrongGroup,
}

impl madsim::net::rpc::Request for Op {
    type Response = Result<Reply, crate::kvraft::msg::Error>;
    const ID: u64 = 4;
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ShardData {
    pub values: std::collections::BTreeMap<String, String>,
    pub replies: std::collections::BTreeMap<u128, (u64, Reply)>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Pull { pub config: u64, pub shard: usize }
impl madsim::net::rpc::Request for Pull {
    type Response = Option<ShardData>;
    const ID: u64 = 20;
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Collect { pub config: u64, pub shard: usize }
impl madsim::net::rpc::Request for Collect {
    type Response = bool;
    const ID: u64 = 21;
}
