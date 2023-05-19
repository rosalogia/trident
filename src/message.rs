#[repr(i32)]
#[derive(Debug)]
pub enum Message {
    KeepAlive = 10,
    Choke = 0,
    Unchoke = 1,
    Interested = 2,
    NotInterested = 3,
    Have(i32) = 4,
    Bitfield(Vec<u8>) = 5,
    Request(i32, i32, i32) = 6,
    Piece(i32, i32, Vec<u8>) = 7,
    Cancel(i32, i32, i32) = 8,
}
