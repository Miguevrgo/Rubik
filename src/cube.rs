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

    fn right<const CW: bool>(&mut self) {
        let e0 = self.edges & 0xF;
        let e4 = (self.edges >> 16) & 0xF;
        let e5 = (self.edges >> 20) & 0xF;
        let e8 = (self.edges >> 32) & 0xF;

        let (ne0, ne4, ne5, ne8) = if CW {
            (e5, e0, e8, e4)
        } else {
            (e4, e8, e0, e5)
        };

        self.edges =
            (self.edges & !0x0000_000F_00FF_000F) | ne0 | (ne4 << 16) | (ne5 << 20) | (ne8 << 32);

        let c0 = self.corners & 0xF;
        let c1 = (self.corners >> 8) & 0xF;
        let c4 = (self.corners >> 32) & 0xF;
        let c5 = (self.corners >> 40) & 0xF;

        let (nc0, nc1, nc4, nc5) = if CW {
            (c1, c5, c0, c4)
        } else {
            (c4, c0, c5, c1)
        };

        self.corners =
            (self.corners & !0x0000_00FF_FF00_FFFF) | nc0 | (nc1 << 8) | (nc4 << 32) | (nc5 << 40);
    }

    fn left<const CW: bool>(&mut self) {
        let e2 = (self.edges >> 8) & 0xF;
        let e6 = (self.edges >> 24) & 0xF;
        let e7 = (self.edges >> 28) & 0xF;
        let e10 = (self.edges >> 40) & 0xF;

        let (ne2, ne6, ne7, ne10) = if CW {
            (e7, e2, e10, e6)
        } else {
            (e6, e10, e2, e7)
        };

        self.edges = (self.edges & !0x0000_0F00_FF00_0F00)
            | (ne2 << 8)
            | (ne6 << 24)
            | (ne7 << 28)
            | (ne10 << 40);

        let c2 = (self.corners >> 16) & 0xFF;
        let c3 = (self.corners >> 24) & 0xFF;
        let c6 = (self.corners >> 48) & 0xFF;
        let c7 = (self.corners >> 56) & 0xFF;

        let (nc2, nc3, nc6, nc7) = if CW {
            (c3, c7, c6, c2)
        } else {
            (c2, c6, c7, c3)
        };

        self.corners = (self.corners & !0x0000_00FF_FF00_FFFF)
            | (nc2 << 16)
            | (nc3 << 24)
            | (nc6 << 48)
            | (nc7 << 56);
    }

    fn front<const CW: bool>(&mut self) {
        let e3 = (self.edges >> 12) & 0xF;
        let e4 = (self.edges >> 16) & 0xF;
        let e7 = (self.edges >> 28) & 0xF;
        let e11 = (self.edges >> 44) & 0xF;

        let (ne3, ne4, ne7, ne11) = if CW {
            (e4, e11, e3, e7)
        } else {
            (e7, e3, e11, e4)
        };

        self.edges = (self.edges & !0x0000_F000_F00F_F000)
            | (ne3 << 12)
            | (ne4 << 16)
            | (ne7 << 28)
            | (ne11 << 44);

        let c0 = self.corners & 0xFF;
        let c3 = (self.corners >> 24) & 0xFF;
        let c4 = (self.corners >> 32) & 0xFF;
        let c7 = (self.corners >> 56) & 0xFF;

        let (nc0, nc3, nc4, nc7) = if CW {
            (c4, c0, c7, c3)
        } else {
            (c3, c7, c0, c4)
        };

        self.corners =
            (self.corners & !0x0000_00FF_FF00_FFFF) | nc0 | (nc3 << 24) | (nc4 << 32) | (nc7 << 56);
    }

    fn back<const CW: bool>(&mut self) {
        let e1 = (self.edges >> 4) & 0xF;
        let e5 = (self.edges >> 20) & 0xF;
        let e6 = (self.edges >> 24) & 0xF;
        let e9 = (self.edges >> 36) & 0xF;

        let (ne1, ne5, ne6, ne9) = if CW {
            (e6, e1, e9, e5)
        } else {
            (e5, e9, e1, e6)
        };

        self.edges = (self.edges & !0x0000_00F0_0FF0_00F0)
            | (ne1 << 4)
            | (ne5 << 20)
            | (ne6 << 24)
            | (ne9 << 36);

        let c1 = (self.corners >> 8) & 0xFF;
        let c2 = (self.corners >> 16) & 0xFF;
        let c5 = (self.corners >> 40) & 0xFF;
        let c6 = (self.corners >> 48) & 0xFF;

        let (nc1, nc2, nc5, nc6) = if CW {
            (c2, c6, c1, c5)
        } else {
            (c5, c1, c6, c2)
        };

        self.corners = (self.corners & !0x00FF_FF00_00FF_FF00)
            | (nc1 << 8)
            | (nc2 << 16)
            | (nc5 << 40)
            | (nc6 << 48);
    }
}
