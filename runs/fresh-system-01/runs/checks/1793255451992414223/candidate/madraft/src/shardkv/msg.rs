use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Op {
    Config { config:crate::shard_ctrler::msg::Config },
    Install { config:u64,shard:usize,data:Transfer },
    Delete { config:u64,shard:usize },
    Ready { config:u64,shard:usize },
    Get { key: String },
    Put { key: String, value: String },
    Append { key: String, value: String },
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

#[derive(Debug,Clone,Default,Serialize,Deserialize)]
pub struct Transfer {pub data:std::collections::HashMap<String,String>,pub sessions:std::collections::HashMap<u64,(u64,Reply)>}
#[derive(Clone,Serialize,Deserialize)]
pub struct Fetch {pub config:u64,pub shard:usize,pub delete:bool}
impl madsim::net::rpc::Request for Fetch {type Response=Option<Transfer>;const ID:u64=12;}
