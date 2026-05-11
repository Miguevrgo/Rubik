/// There are 12 edges, numbered anticlockwise starting from 6 always
/// This numeration fits into 4 bits which are also clustered inside
/// an u64 as we need 4 * 12 = 48 bits
///
/// There are 8 corners, we use 8 bits for each even if we don't need
/// that much information so that we have 8-bit pretty numbers, numbered
/// from top to bottom anticlockwise totaling 64 bits
pub struct Cube {
    edges: u64,
    corners: u64,
}

impl Cube {
    pub fn new() -> Self {
        Self {
            edges: 0xBA98_7654_3210,
            corners: 0x0706_0504_0302_0100,
        }
    }

    /// Eeach possible move (clockwise or anticlockwise)
    fn up<const CW: bool>(&mut self) {
        let tmp_11_pack = self.edges & 0x1000_0000_0000;
        let tmp_3_pack = self.edges & 0x100;
        self.edges = self.edges.rotate_left(4);
        self.edges &= 0x1111_0111_1111_1111;
        self.edges &= 0x1111_1111_1111_1011;
    }
    fn down<const CW: bool>(&mut self) {}
    fn right<const CW: bool>(&mut self) {}
    fn left<const CW: bool>(&mut self) {}
    fn top<const CW: bool>(&mut self) {}
    fn bottom<const CW: bool>(&mut self) {}
}
