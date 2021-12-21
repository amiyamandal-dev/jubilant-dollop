use hyper::body::Bytes;
use serde::{Deserialize, Serialize};
use serde_bencode::{from_bytes, to_bytes};
use super::peers::Peer;

#[derive(Debug, Deserialize, Serialize)]
pub struct TrackerResp {
    #[serde(default)]
    #[serde(rename = "interval")]
    interval: Option<i64>,
    #[serde(default)]
    #[serde(rename = "peers")]
    peers: serde_bytes::ByteBuf,
    #[serde(default)]
    #[serde(rename = "complete")]
    complete: Option<i64>,
    #[serde(default)]
    #[serde(rename = "incomplete")]
    incomplete: Option<i64>,
}

impl TrackerResp {
    pub fn new(buffer: Bytes) -> Self {
        let mut t = match from_bytes::<TrackerResp>(&buffer) {
            Ok(t) => t,
            Err(e) => panic!("{:?}", e),
        };
        let _ = Peer::new(t.peers.clone());
        t
    }
}
