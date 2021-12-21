use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Hash)]
pub struct Peer {
    IP: String,
    Port:i64,
}