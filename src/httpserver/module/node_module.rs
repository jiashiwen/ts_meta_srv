use serde::{Deserialize, Serialize};

pub const NODE_ID_PREFIX: &'static str = "nodeid_";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeInfo {
    pub id: u64,
    pub grpc_addr: String,
    pub http_addr: String,
    pub attribute: String,
}
