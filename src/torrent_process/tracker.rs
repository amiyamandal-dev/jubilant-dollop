use super::peers::Peer;
use hyper::body::Bytes;
use serde::{Deserialize, Serialize};
use serde_bencode::from_bytes;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct GenericError(String);

impl fmt::Display for GenericError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "There is an error: {}", self.0)
    }
}

impl Error for GenericError {}

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
    actual_peers: Option<Vec<Peer>>,
}

impl TrackerResp {
    pub fn new(buffer: Bytes) -> Result<Self, Box<dyn Error>> {
        let mut t = match from_bytes::<TrackerResp>(&buffer) {
            Ok(t) => t,
            Err(e) => return Err(Box::new(GenericError(format!("{}", e.to_string())))),
        };
        let p = match Peer::new(t.peers.clone()) {
            Ok(t) => t,
            Err(e) => return Err(Box::new(GenericError(format!("{}", e.to_string())))),
        };
        t.actual_peers = Some(p);
        Ok(t)
    }
}
