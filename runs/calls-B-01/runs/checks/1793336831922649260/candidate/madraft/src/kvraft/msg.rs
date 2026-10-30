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

/// Stable identity across RPC retries. A client serializes its sequence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientRequest<T> { pub client: u64, pub sequence: u64, pub command: T }
impl<T: madsim::net::rpc::Request> madsim::net::rpc::Request for ClientRequest<T> {
    type Response = T::Response;
    const ID: u64 = T::ID + 10000;
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Replicated { pub client: u64, pub sequence: u64, pub command: Vec<u8> }
