use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Op {
    Get { key: String },
    Put { key: String, value: String },
    Append { key: String, value: String },
}

#[derive(thiserror::Error, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Error {
    #[error("not leader, hint: {hint}")]
    NotLeader { hint: usize },
    #[error("timeout")]
    Timeout,
    #[error("failed to reach consensus")]
    Failed,
}

impl madsim::net::rpc::Request for Op {
    type Response = Result<String, crate::kvraft::msg::Error>;
    const ID: u64 = 2;
}

/// Retries carry the same identity through the Raft log and snapshots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientRequest<R> {
    pub client: u128,
    pub sequence: u64,
    pub command: R,
}
impl<R: madsim::net::rpc::Request> madsim::net::rpc::Request for ClientRequest<R> {
    type Response = R::Response;
    const ID: u64 = R::ID + 100;
}
