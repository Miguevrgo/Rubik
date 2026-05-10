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
            edges: ,
            corners: (),
        }
    }
}
