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
    pub fn up<const CW: bool>(&mut self) {
        let edges_face = (self.edges & 0xFFFF) as u16;
        let rotated_edges = if CW {
            edges_face.rotate_left(4)
        } else {
            edges_face.rotate_right(4)
        };
        self.edges = (self.edges & !0xFFFF) | (rotated_edges as u64);

        let corners_face = (self.corners & 0xFFFF_FFFF) as u32;
        let rotated_corners = if CW {
            corners_face.rotate_left(8)
        } else {
            corners_face.rotate_right(8)
        };
        self.corners = (self.corners & !0xFFFF_FFFF) | (rotated_corners as u64);
    }

    fn down<const CW: bool>(&mut self) {
        let edges_face = (self.edges >> 32) as u16;
        let rotated_edges = if CW {
            edges_face.rotate_right(4)
        } else {
            edges_face.rotate_left(4)
        };
        self.edges = (self.edges & !0xFFFF_0000_0000) | ((rotated_edges as u64) << 32);

        let corners_face = (self.corners >> 32) as u32;
        let rotated_corners = if CW {
            corners_face.rotate_right(8)
        } else {
            corners_face.rotate_left(8)
        };
        self.corners = (self.corners & !0xFFFF_FFFF_0000_0000) | ((rotated_corners as u64) << 32);
    }

    fn right<const CW: bool>(&mut self) {}
    fn left<const CW: bool>(&mut self) {}
    fn top<const CW: bool>(&mut self) {}
    fn bottom<const CW: bool>(&mut self) {}
}
