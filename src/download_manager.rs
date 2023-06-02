use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::io::{self, IoSlice};
use std::sync::mpsc;
use std::thread;

use crate::peers::Peer;
use crate::tracker_communication::PeerInfo;
use crate::BLOCK_SIZE;
use crate::{bitfield::Bitfield, metainfo::Torrent};

pub struct PieceProgress {
    pub index: u64,
    pub blocks: Vec<Vec<u8>>,
}

impl PieceProgress {
    pub fn new(index: u64, piece_length: u64) -> Self {
        let blocks_in_piece = (piece_length as f64 / BLOCK_SIZE as f64).ceil() as usize;
        // let blocks_in_piece = (piece_length / BLOCK_SIZE) as usize;
        PieceProgress {
            index,
            blocks: vec![vec![0; BLOCK_SIZE as usize]; blocks_in_piece],
        }
    }
}

pub struct DownloadManager {
    pub torrent: Torrent,
    pub work_queue: Vec<PieceProgress>,
    pub pieces: Bitfield,
    file: File,
}

impl DownloadManager {
    pub fn from(torrent: Torrent) -> std::io::Result<Self> {
        let num_pieces = torrent.info.pieces.len() / 20;
        let work_queue = (0..num_pieces)
            .map(|i| PieceProgress::new(i as u64, torrent.info.piece_length))
            .collect();
        let pieces = Bitfield::empty(num_pieces);
        let file = std::fs::File::create(torrent.info.name.clone())?;
        Ok(DownloadManager {
            torrent,
            work_queue,
            pieces,
            file,
        })
    }

    fn verify_piece(&self, piece: &PieceProgress) -> bool {
        let offset = (piece.index * 20) as usize;
        let piece_hash = &self.torrent.info.pieces[offset..offset + 20];
        let mut hasher = Sha1::new();
        piece.blocks.iter().for_each(|b| hasher.update(b));
        let hash = hasher.finalize().to_vec();
        hash == piece_hash
    }

    pub fn submit_piece(&mut self, piece: PieceProgress) -> std::io::Result<usize> {
        if !self.verify_piece(&piece) {
            return Ok(0);
        }

        self.pieces.set_piece(&piece.index);

        let offset = piece.index as u64 * self.torrent.info.piece_length;
        self.file.seek(SeekFrom::Start(offset))?;
        let slices: Vec<IoSlice<'_>> = piece
            .blocks
            .iter()
            .map(|block| IoSlice::new(&block))
            .collect();
        self.file.write_vectored(&slices)
    }

    pub fn download_pieces(&mut self, mut peers: Vec<PeerInfo>) -> std::io::Result<()> {
        let (tx, rx) = mpsc::channel();

        for i in 0..4 {
            let _tx = tx.clone();
            let peer = peers.pop();
            let work = self.work_queue.pop();
            let torrent_info = self.torrent.info.clone();
            if let Some(peer) = peer {
                thread::spawn(move || {
                    let mut work = work.unwrap();
                    if let Ok(mut peer) = Peer::new(peer, torrent_info) {
                        let peer_id: &str = "-TD1000-111111111111";
                        peer.handshake(&peer.torrent_info.info_hash(), peer_id)
                            .unwrap();
                        peer.request_piece(&mut work).unwrap();
                        _tx.send(work).unwrap()
                    }
                });
            }
        }

        for i in 0..4 {
            let received = rx.recv().unwrap();
            // println!("{:?}", received.blocks);
            println!("Valid piece: {}", self.verify_piece(&received));
        }

        Ok(())
    }
}
