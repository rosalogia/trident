use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::io::{self, IoSlice};

use crate::peers::Peer;
use crate::{bitfield::Bitfield, metainfo::Torrent};

pub struct PieceProgress {
    pub index: usize,
    pub blocks: Vec<Vec<u8>>,
}

impl PieceProgress {
    pub fn new(index: usize, piece_length: u64) -> Self {
        let blocks_in_piece = (piece_length / 16384) as usize;
        PieceProgress {
            index,
            blocks: vec![vec![0; 16384]; blocks_in_piece],
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
            .map(|i| PieceProgress::new(i, torrent.info.piece_length))
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
        let offset = piece.index * 20;
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

}
