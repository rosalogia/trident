pub struct Bitfield(pub Vec<u8>);

impl Bitfield {
    pub fn empty(pieces: usize) -> Self {
        let entries = (pieces as f64 / 8 as f64).ceil() as usize;
        Bitfield(vec![0; entries])
    }

    pub fn from(pieces: Vec<u8>) -> Self {
        Bitfield(pieces)
    }

    pub fn has_piece(&self, index: &usize) -> bool {
        (self.0[index / 8] & (1 << (index % 8))) == 1
    }

    pub fn set_piece(&mut self, index: &usize) -> () {
        self.0[index / 8] = self.0[index / 8] | (1 << (index % 8))
    }

}
