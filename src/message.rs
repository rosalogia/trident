#[repr(u8)]
#[derive(Debug)]
pub enum Message {
    KeepAlive = 10,
    Choke = 0,
    Unchoke = 1,
    Interested = 2,
    NotInterested = 3,
    Have(u64) = 4,
    Bitfield(Vec<u8>) = 5,
    Request(u64, u64, u64) = 6,
    Piece(u64, u64, Vec<u8>) = 7,
    Cancel(u64, u64, u64) = 8,
}

impl Message {
    pub fn as_bytes(&self) -> Vec<u8> {
        use Message::*;
        let discriminant = unsafe { *(self as *const Self as *const u8) };

        match self {
            KeepAlive => 0_i32.to_be_bytes().to_vec(),
            Choke | Unchoke | Interested | NotInterested => vec![0, 0, 0, 1, discriminant],
            Have(index) => vec![0, 0, 0, 5, 4, *index as u8],
            Bitfield(bitfield) => {
                let length = (1 + bitfield.len()).to_be_bytes().to_vec();
                [length, vec![5], bitfield.clone()].concat()
            }
            Request(index, begin, length) | Cancel(index, begin, length) => [
                13_i32.to_be_bytes().to_vec(),
                vec![discriminant],
                index.to_be_bytes().to_vec(),
                begin.to_be_bytes().to_vec(),
                length.to_be_bytes().to_vec(),
            ]
            .concat(),
            Piece(index, begin, block) => {
                let length = 9 + block.len();
                [
                    length.to_be_bytes().to_vec(),
                    vec![7],
                    index.to_be_bytes().to_vec(),
                    begin.to_be_bytes().to_vec(),
                    block.clone(),
                ]
                .concat()
            }
        }
    }
}
