use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr};

use super::PEERSIZE;

#[derive(Debug)]
pub struct GenericError(String);

impl fmt::Display for GenericError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "There is an error: {}", self.0)
    }
}

impl Error for GenericError {}

#[derive(Debug, Deserialize, Serialize)]
pub struct Peer {
    pub IP: Ipv4Addr,
    pub Port: u16,
}

impl Peer {
    pub fn new(peers_blob: serde_bytes::ByteBuf) -> Result<Vec<Self>, Box<dyn Error>> {
        let mut p: Vec<Peer> = vec![];
        if peers_blob.len() % PEERSIZE != 0 {
            return Err(Box::new(GenericError(
                format!("Received malformed pieces of length {}", peers_blob.len()).into(),
            )));
        }
        let num_peers = peers_blob.len() / PEERSIZE;
        for i in 0..num_peers {
            let offset = i * PEERSIZE;
            let ip_v4 = bip_util::convert::bytes_be_to_ipv4(
                peers_blob[offset..offset + 4]
                    .try_into()
                    .expect("slice with incorrect length"),
            );
            let port = bip_util::convert::bytes_be_to_port(
                peers_blob[offset + 4..offset + 6]
                    .try_into()
                    .expect("slice with incorrect length"),
            );
            let temp_peer = Peer {
                IP: ip_v4,
                Port: port,
            };
            p.push(temp_peer);
        }
        Ok(p)
    }
}
