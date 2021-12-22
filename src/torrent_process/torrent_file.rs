extern crate serde;
extern crate serde_bencode;
extern crate serde_derive;

use futures::future::ok;
use serde::{Deserialize, Serialize};
use serde_bencode::{de, to_bytes};
use serde_bytes::ByteBuf;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::hash::Hash;
use tokio::fs::File;
use tokio::io::{self, AsyncReadExt};
use url::Url;
use urlencoding::{decode, decode_binary, encode, encode_binary};

use super::peers::Peer;
use super::tracker;
use super::utils::generate_id;
use super::Port;

#[derive(Debug)]
pub struct GenericError(String);

impl fmt::Display for GenericError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "There is an error: {}", self.0)
    }
}

impl Error for GenericError {}

#[derive(Debug, Deserialize, Serialize, Hash)]
struct FileType {
    path: Vec<String>,
    length: i64,
    #[serde(default)]
    md5sum: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Hash)]
struct Info {
    name: String,
    pieces: ByteBuf,
    #[serde(rename = "piece length")]
    piece_length: i64,
    #[serde(default)]
    md5sum: Option<String>,
    #[serde(default)]
    length: Option<i64>,
    #[serde(default)]
    files: Option<Vec<FileType>>,
    #[serde(default)]
    private: Option<u8>,
    #[serde(default)]
    path: Option<Vec<String>>,
    #[serde(default)]
    #[serde(rename = "root hash")]
    root_hash: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Hash)]
struct TorrentFile {
    info: Info,
    #[serde(default)]
    announce: Option<String>,
    #[serde(default)]
    encoding: Option<String>,
    #[serde(default)]
    httpseeds: Option<Vec<String>>,
    #[serde(default)]
    #[serde(rename = "announce-list")]
    announce_list: Option<Vec<Vec<String>>>,
    #[serde(default)]
    #[serde(rename = "creation date")]
    creation_date: Option<i64>,
    #[serde(rename = "comment")]
    comment: Option<String>,
    #[serde(default)]
    #[serde(rename = "created by")]
    created_by: Option<String>,
}

impl TorrentFile {
    pub async fn new(file_name_with_path: String) -> io::Result<Self> {
        let mut f = File::open(file_name_with_path).await?;
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer).await?;
        let mut t = match de::from_bytes::<TorrentFile>(&buffer) {
            Ok(t) => t,
            Err(e) => panic!("{:?}", e),
        };
        Ok(t)
    }
    pub fn id(&self) -> Vec<u8> {
        let mut hasher = Sha1::new();
        let bencode_byte = match to_bytes(&self.info) {
            Ok(t) => t,
            Err(e) => panic!("{:?}", e),
        };
        hasher.update(bencode_byte);
        let result: Vec<u8> = hasher.finalize().to_vec();
        result
    }
    pub fn split_piece_hashes(&self) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
        let hash_len = 20;
        let mut temp_pieces = self.info.pieces.to_vec();
        let mut piece_hashes: Vec<Vec<u8>> = Vec::new();
        if self.info.pieces.len() % hash_len != 0 {
            return Err(Box::new(GenericError(
                format!(
                    "Received malformed pieces of length {}",
                    self.info.pieces.len()
                )
                .into(),
            )));
        }
        for chunk in temp_pieces.chunks(hash_len) {
            piece_hashes.push(chunk.to_vec());
        }

        Ok(piece_hashes)
    }
}

pub struct ProcessTorrent {
    t: TorrentFile,
    info_hash: Vec<u8>,
    piece_hashes: Vec<Vec<u8>>,
    piece_length: i64,
    peer_id: Vec<u8>,
}

impl ProcessTorrent {
    pub async fn new(file_name_with_path: String) -> Result<Self, Box<dyn Error>> {
        let t = match TorrentFile::new(file_name_with_path.clone()).await {
            Ok(t) => t,
            Err(e) => {
                return Err(Box::new(GenericError(format!(
                    "error occur while processing {} -> {}",
                    file_name_with_path, e
                ))));
            }
        };
        let piece_hashes = match t.split_piece_hashes() {
            Ok(t) => t,
            Err(e) => {
                return Err(Box::new(GenericError(format!(
                    "error occur while processing {} -> {}",
                    file_name_with_path, e
                ))));
            }
        };
        let piece_lenght = t.info.piece_length;
        let info_hash = t.id();
        let p = ProcessTorrent {
            t,
            info_hash: info_hash,
            piece_hashes,
            piece_length: piece_lenght,
            peer_id: generate_id(),
        };
        Ok(p)
    }

    pub fn build_tracker_URL(&self) -> Result<String, Box<dyn Error>> {
        let announce = match &self.t.announce {
            Some(t) => t,
            None => return Err(Box::new(GenericError(format!("No announce url exits")))),
        };
        let info_hash = encode_binary(&self.info_hash);
        let mut peer_id = encode_binary(&self.peer_id);

        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("port".to_string(), Port.to_string());
        params.insert("uploaded".to_string(), "0".to_string());
        params.insert("downloaded".to_string(), "0".to_string());
        params.insert("compact".to_string(), "1".to_string());
        params.insert("left".to_string(), self.piece_length.to_string());
        params.insert("info_hash".to_string(), info_hash.to_string());
        params.insert("peer_id".to_string(), peer_id.to_string());

        let url = match Url::parse_with_params(&announce, params) {
            Ok(t) => t,
            Err(e) => return Err(Box::new(GenericError(format!("{}", e.to_string())))),
        };
        let url_str = url.to_string();
        let final_url = match decode(&url_str) {
            Ok(t) => t,
            Err(e) => {
                return Err(Box::new(GenericError(format!(
                    "unable to genrate url {}",
                    e
                ))))
            }
        };
        Ok(final_url.to_string())
    }

    pub async fn request_peers(&self) -> Result<(), Box<dyn Error>> {
        let url = match self.build_tracker_URL() {
            Ok(t) => t,
            Err(e) => return Err(Box::new(GenericError(format!("{}", e.to_string())))),
        };
        let resp = match reqwest::get(url).await {
            Ok(t) => t,
            Err(e) => return Err(Box::new(GenericError(format!("{}", e.to_string())))),
        };
        let byte_response = match resp.bytes().await {
            Ok(t) => t,
            Err(e) => return Err(Box::new(GenericError(format!("{}", e.to_string())))),
        };
        // println!("{:x?}", byte_response);
        let t = tracker::TrackerResp::new(byte_response);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_code() {
        let mut t = ProcessTorrent::new("ubuntu-21.10-desktop-amd64.iso.torrent".to_string())
            .await
            .unwrap();
        let url = t.build_tracker_URL().unwrap();
        println!("{:?}", url);
        t.request_peers().await.unwrap();
    }
}
