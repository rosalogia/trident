use crate::bitfield::Bitfield;
use crate::download_manager::PieceProgress;
use crate::message::Message;
use crate::metainfo::Info;
use crate::tracker_communication::PeerInfo;
use crate::BLOCK_SIZE;
use std::cmp::min;
use std::io::prelude::*;
use std::net::TcpStream;
use std::str;

#[derive(Debug)]
pub struct HandShake {
    pub protocol: String,
    pub reserved: Vec<u8>,
    pub info_hash: Vec<u8>,
    pub peer_id: Vec<u8>,
}

impl HandShake {
    pub fn as_bytes(&self) -> Vec<u8> {
        let protocol_length = self.protocol.len();
        [
            &[protocol_length as u8],
            self.protocol.as_bytes(),
            &self.reserved,
            &self.info_hash,
            &self.peer_id,
        ]
        .concat()
    }

    pub fn from_bytes(payload: Vec<u8>) -> Result<Self, String> {
        if payload.len() < 48 {
            return Err(String::from(
                "Handshake is invalid; message not long enough",
            ));
        }

        let mut cursor = 0;
        let protocol_length = payload[cursor] as usize;
        cursor += 1;
        let protocol = match str::from_utf8(&payload[cursor..protocol_length + cursor]) {
            Ok(name) => String::from(name),
            Err(e) => {
                return Err(format!("{}", e));
            }
        };
        cursor += protocol_length;
        let reserved = payload[cursor..cursor + 8].to_vec();
        cursor += 8;
        let info_hash = payload[cursor..cursor + 20].to_vec();
        cursor += 20;
        let peer_id = payload[cursor..cursor + 20].to_vec();
        Ok(HandShake {
            protocol,
            reserved,
            info_hash,
            peer_id,
        })
    }
}

pub struct Peer {
    pub am_choking: bool,
    pub am_interested: bool,
    pub is_choking: bool,
    pub is_interested: bool,
    pub address: String,
    pub peer_id: Option<String>,
    pub connection: TcpStream,
    pub torrent_info: Info,
    pub bit_field: Option<Bitfield>,
    pub message_backlog: Vec<Message>,
}

impl Peer {
    pub fn new(peer_info: PeerInfo, torrent_info: Info) -> std::io::Result<Self> {
        let address = format!("{}:{}", peer_info.ip, peer_info.port);
        let stream = TcpStream::connect(&address)?;
        Ok(Peer {
            am_choking: true,
            am_interested: false,
            is_choking: true,
            is_interested: false,
            address,
            peer_id: None,
            connection: stream,
            torrent_info,
            bit_field: None,
            message_backlog: vec![],
        })
    }

    fn int_of_bytes(bytes: &[u8; 4]) -> u64 {
        ((bytes[0] as u64) << 24 as u64)
            + ((bytes[1] as u64) << 16 as u64)
            + ((bytes[2] as u64) << 8 as u64)
            + (bytes[3] as u64)
    }

    fn next_int(&mut self) -> std::io::Result<u64> {
        let mut buff = [0; 4];
        self.connection.read(&mut buff)?;
        Ok(Self::int_of_bytes(&buff))
    }

    pub fn next_message(&mut self) -> std::io::Result<Message> {
        let msg_size = self.next_int()?;
        let mut msg_type = [0];
        self.connection.read(&mut msg_type)?;
        Ok(match msg_type[0] {
            0 => Message::Choke,
            1 => Message::Unchoke,
            2 => Message::Interested,
            3 => Message::NotInterested,
            4 => {
                let index = self.next_int()?;
                self.set_piece(&index);
                Message::Have(index)
            }
            5 => {
                let mut buff = vec![0; (msg_size - 1) as usize];
                self.connection.read(&mut buff)?;
                self.bit_field = Some(Bitfield::from(buff.clone()));
                Message::Bitfield(buff)
            }
            6 => {
                let index = self.next_int()?;
                let begin = self.next_int()?;
                let length = self.next_int()?;
                Message::Request(index, begin, length)
            }
            7 => {
                let index = self.next_int()?;
                let begin = self.next_int()?;
                let mut block = vec![0; (msg_size - 9) as usize];
                self.connection.read(&mut block)?;
                Message::Piece(index, begin, block)
            }
            8 => {
                let index = self.next_int()?;
                let begin = self.next_int()?;
                let length = self.next_int()?;
                Message::Cancel(index, begin, length)
            }
            _ => Message::KeepAlive,
        })
    }

    pub fn handshake(&mut self, info_hash: &[u8], peer_id: &str) -> std::io::Result<HandShake> {
        let handshake = HandShake {
            protocol: String::from("BitTorrent protocol"),
            reserved: vec![0; 8],
            info_hash: info_hash.to_owned(),
            peer_id: peer_id.as_bytes().to_owned(),
        };

        println!("{:?}", handshake.as_bytes());
        self.connection.write(&handshake.as_bytes())?;
        let mut buff = vec![0; 68];
        self.connection.read(&mut buff)?;
        Ok(HandShake::from_bytes(buff).unwrap())
    }

    fn set_piece(&mut self, piece: &u64) -> () {
        if let Some(bitfield) = &mut self.bit_field {
            bitfield.set_piece(piece)
        }
    }

    pub fn has_piece(&self, piece: &u64) -> Option<bool> {
        self.bit_field.as_ref().map(|bf| bf.has_piece(piece))
    }

    fn send_message(&mut self, message: Message) -> std::io::Result<usize> {
        self.connection.write(&message.as_bytes())
    }

    pub fn request_block(&mut self, piece: u64, begin: u64) -> std::io::Result<()> {
        let size = if begin + BLOCK_SIZE > self.torrent_info.piece_length {
            self.torrent_info.piece_length % BLOCK_SIZE
        } else {
            BLOCK_SIZE
        };

        self.send_message(Message::Request(piece, begin, size))?;
        Ok(())
    }

    pub fn request_piece(&mut self, piece_progress: &mut PieceProgress) -> std::io::Result<()> {
        let blocks = (self.torrent_info.piece_length as f64 / BLOCK_SIZE as f64).ceil() as u64;
        for i in 0..blocks {
            self.request_block(piece_progress.index, i * BLOCK_SIZE)?;
        }

        for i in 0..blocks {
            let next_message = self.next_message()?;
            match next_message {
                Message::Piece(index, begin, contents) => {
                    if index == piece_progress.index {
                        let block_index = (begin / BLOCK_SIZE) as usize;
                        let reach = min(BLOCK_SIZE as usize, contents.len());
                        for j in 0..reach {
                            piece_progress.blocks[block_index][j] = contents[j];
                        }
                    }
                }
                m => {
                    self.message_backlog.push(m);
                }
            }
        }

        Ok(())
    }
}
