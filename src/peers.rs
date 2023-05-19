use std::io::prelude::*;
use std::net::TcpStream;
use crate::metainfo::Info;

pub struct Peer {
    pub am_choking: bool,
    pub am_interested: bool,
    pub is_choking: bool,
    pub is_interested: bool,
    pub address: String,
    pub peer_id: Option<String>,
    pub connection: TcpStream,
    pub torrent_info: Info
}

impl Peer {
    pub fn new(address: String, torrent_info: Info) -> std::io::Result<Self> {
        let stream = TcpStream::connect(&address)?;
        Ok(Peer {
            am_choking: true,
            am_interested: false,
            is_choking: true,
            is_interested: false,
            address,
            peer_id: None,
            connection: stream,
            torrent_info
        })
    }

    pub fn handshake(&mut self, info_hash: &[u8], peer_id: &str) -> std::io::Result<[u8; 68]> {
        let handshake_msg = [
            "\x13BitTorrent protocol\x00\x00\x00\x00\x00\x00\x00\x00".as_bytes(),
            info_hash,
            peer_id.as_bytes(),
        ]
        .concat();
        println!("{:?}", handshake_msg);
        self.connection.write(&handshake_msg)?;
        let mut buff = [0; 68];
        self.connection.read(&mut buff)?;
        Ok(buff)
    }

    pub fn get_bitfield(&mut self) -> std::io::Result<Vec<u8>> {
        let mut size_header = [0; 4];
        self.connection.read(&mut size_header)?;
        let size = ((size_header[0] as u64) << 24 as u64) + ((size_header[1] as u64) << 16 as u64) + ((size_header[2] as u64) << 8 as u64) + ((size_header[3] as u64));
        let mut buff = vec![0; size as usize];
        self.connection.read(&mut buff)?;
        Ok(buff)
    }
}
