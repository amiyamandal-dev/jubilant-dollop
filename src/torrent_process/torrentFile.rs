extern crate serde;
extern crate serde_bencode;
extern crate serde_derive;

use futures::future::err;
use hex_literal::hex;
use serde::{Deserialize, Serialize};
use serde_bencode::{de, to_bytes};
use serde_bytes::ByteBuf;
use sha1::{Digest, Sha1};
use std::error::Error;
use std::fmt;
use std::hash::{Hash, Hasher};
use tokio::fs::File;
use tokio::io::{self, AsyncReadExt};

#[derive(Debug)]
pub struct GenericError(String);

impl fmt::Display for GenericError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "There is an error: {}", self.0)
    }
}

impl Error for GenericError {}

#[derive(Debug, Deserialize, Serialize, Hash)]
struct Node(String, i64);

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
    nodes: Option<Vec<Node>>,
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
        // println!("{}", result.len());
        // let r = format!("{:x}", result);
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
        for chunk in temp_pieces.chunks(hash_len){
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
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_code() {
        let t = TorrentFile::new("archlinux-2019.12.01-x86_64.iso.torrent".to_string())
            .await
            .unwrap();
        println!("{:?}", t.info.root_hash);
        let r = t.id();
        println!("{}", r.len());
        let k = t.split_piece_hashes().unwrap();
        println!("{:X?}", k[0]);
    }
}
