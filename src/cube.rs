/// There are 12 edges, numbered anticlockwise starting from 6 always
/// This numeration fits into 4 bits which are also clustered inside
/// an u64 as we need 4 * 12 = 48 bits, then we use next 12 bits for
/// the orientation
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
            edges: 0x0FFF_BA98_7654_3210,
            corners: 0x0706_0504_0302_0100,
        }
    }

    /// Eeach possible move (clockwise or anticlockwise)
    pub fn up<const PRIME: bool>(&mut self) {
        let edges_face = (self.edges & 0xFFFF) as u16;
        let rotated_edges = if PRIME {
            edges_face.rotate_left(4)
        } else {
            edges_face.rotate_right(4)
        };
        self.edges = (self.edges & !0xFFFF) | (rotated_edges as u64);

        let corners_face = (self.corners & 0xFFFF_FFFF) as u32;
        let rotated_corners = if PRIME {
            corners_face.rotate_left(8)
        } else {
            corners_face.rotate_right(8)
        };
        self.corners = (self.corners & !0xFFFF_FFFF) | (rotated_corners as u64);

        // Orientation
        let eo_bits = ((self.edges >> 48) & 0xF) as u8;
        let rotated_eo = if PRIME {
            ((eo_bits << 1) & 0xF) | (eo_bits >> 3)
        } else {
            (eo_bits >> 1) | ((eo_bits & 1) << 3)
        };
        self.edges = (self.edges & !0x000F_0000_0000_0000) | ((rotated_eo as u64) << 48);

        // TODO: Corners
    }

    pub fn down<const PRIME: bool>(&mut self) {
        let edges_face = (self.edges >> 32) as u16;
        let rotated_edges = if PRIME {
            edges_face.rotate_right(4)
        } else {
            edges_face.rotate_left(4)
        };
        self.edges = (self.edges & !0xFFFF_0000_0000) | ((rotated_edges as u64) << 32);

        let corners_face = (self.corners >> 32) as u32;
        let rotated_corners = if PRIME {
            corners_face.rotate_right(8)
        } else {
            corners_face.rotate_left(8)
        };
        self.corners = (self.corners & !0xFFFF_FFFF_0000_0000) | ((rotated_corners as u64) << 32);

        // Orientation
        let eo_bits = ((self.edges >> 56) & 0xF) as u8;
        let rotated_eo = if PRIME {
            (eo_bits >> 1) | ((eo_bits & 1) << 3)
        } else {
            ((eo_bits << 1) & 0xF) | (eo_bits >> 3)
        };
        self.edges = (self.edges & !0x0F00_0000_0000_0000) | ((rotated_eo as u64) << 56);
        // TODO: Corners
    }

    pub fn right<const PRIME: bool>(&mut self) {
        let e0 = self.edges & 0xF;
        let e4 = (self.edges >> 16) & 0xF;
        let e5 = (self.edges >> 20) & 0xF;
        let e8 = (self.edges >> 32) & 0xF;

        let (ne0, ne4, ne5, ne8) = if PRIME {
            (e5, e0, e8, e4)
        } else {
            (e4, e8, e0, e5)
        };

        self.edges =
            (self.edges & !0x0000_000F_00FF_000F) | ne0 | (ne4 << 16) | (ne5 << 20) | (ne8 << 32);

        let c0 = self.corners & 0xFF;
        let c1 = (self.corners >> 8) & 0xFF;
        let c4 = (self.corners >> 32) & 0xFF;
        let c5 = (self.corners >> 40) & 0xFF;

        let (nc0, nc1, nc4, nc5) = if PRIME {
            (c1, c5, c0, c4)
        } else {
            (c4, c0, c5, c1)
        };

        self.corners =
            (self.corners & !0x0000_FFFF_0000_FFFF) | nc0 | (nc1 << 8) | (nc4 << 32) | (nc5 << 40);

        // Orientation
        let eo0 = (self.edges >> 48) & 1;
        let eo4 = (self.edges >> 52) & 1;
        let eo5 = (self.edges >> 53) & 1;
        let eo8 = (self.edges >> 56) & 1;
        let (ne_eo0, ne_eo4, ne_eo5, ne_eo8) = if PRIME {
            (eo5, eo0, eo8, eo4)
        } else {
            (eo4, eo8, eo0, eo5)
        };
        self.edges = (self.edges & !0x0131_0000_0000_0000)
            | (ne_eo0 << 48)
            | (ne_eo4 << 52)
            | (ne_eo5 << 53)
            | (ne_eo8 << 56);
        // TODO: Corners
    }

    pub fn left<const PRIME: bool>(&mut self) {
        let e2 = (self.edges >> 8) & 0xF;
        let e6 = (self.edges >> 24) & 0xF;
        let e7 = (self.edges >> 28) & 0xF;
        let e10 = (self.edges >> 40) & 0xF;

        let (ne2, ne6, ne7, ne10) = if PRIME {
            (e6, e10, e2, e7)
        } else {
            (e7, e2, e10, e6)
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

        let (nc2, nc3, nc6, nc7) = if PRIME {
            (c3, c7, c2, c6)
        } else {
            (c6, c2, c7, c3)
        };

        self.corners = (self.corners & !0xFFFF_0000_FFFF_0000)
            | (nc2 << 16)
            | (nc3 << 24)
            | (nc6 << 48)
            | (nc7 << 56);

        // Orientation
        let eo2 = (self.edges >> 50) & 1;
        let eo6 = (self.edges >> 54) & 1;
        let eo7 = (self.edges >> 55) & 1;
        let eo10 = (self.edges >> 58) & 1;
        let (ne_eo2, ne_eo6, ne_eo7, ne_eo10) = if PRIME {
            (eo6, eo10, eo2, eo7)
        } else {
            (eo7, eo2, eo10, eo6)
        };
        self.edges = (self.edges & !0x04C4_0000_0000_0000)
            | (ne_eo2 << 50)
            | (ne_eo6 << 54)
            | (ne_eo7 << 55)
            | (ne_eo10 << 58);
        // TODO: Corners
    }

    pub fn front<const PRIME: bool>(&mut self) {
        let e3 = (self.edges >> 12) & 0xF;
        let e4 = (self.edges >> 16) & 0xF;
        let e6 = (self.edges >> 24) & 0xF;
        let e11 = (self.edges >> 44) & 0xF;

        let (ne3, ne4, ne6, ne11) = if PRIME {
            (e4, e11, e3, e6)
        } else {
            (e6, e3, e11, e4)
        };

        self.edges = (self.edges & !0x0000_F000_0F0F_F000)
            | (ne3 << 12)
            | (ne4 << 16)
            | (ne6 << 24)
            | (ne11 << 44);

        let c0 = self.corners & 0xFF;
        let c3 = (self.corners >> 24) & 0xFF;
        let c4 = (self.corners >> 32) & 0xFF;
        let c7 = (self.corners >> 56) & 0xFF;

        let (nc0, nc3, nc4, nc7) = if PRIME {
            (c4, c0, c7, c3)
        } else {
            (c3, c7, c0, c4)
        };

        self.corners =
            (self.corners & !0xFF00_00FF_FF00_00FF) | nc0 | (nc3 << 24) | (nc4 << 32) | (nc7 << 56);

        // Orientation (with flip)
        let eo3 = (self.edges >> 51) & 1;
        let eo4 = (self.edges >> 52) & 1;
        let eo6 = (self.edges >> 54) & 1;
        let eo11 = (self.edges >> 59) & 1;
        let (ne_eo3, ne_eo4, ne_eo6, ne_eo11) = if PRIME {
            (eo4 ^ 1, eo11 ^ 1, eo3 ^ 1, eo6 ^ 1)
        } else {
            (eo6 ^ 1, eo3 ^ 1, eo11 ^ 1, eo4 ^ 1)
        };
        self.edges = (self.edges & !0x0858_0000_0000_0000)
            | (ne_eo3 << 51)
            | (ne_eo4 << 52)
            | (ne_eo6 << 54)
            | (ne_eo11 << 59);
        // TODO: Corners
    }

    pub fn back<const PRIME: bool>(&mut self) {
        let e1 = (self.edges >> 4) & 0xF;
        let e5 = (self.edges >> 20) & 0xF;
        let e7 = (self.edges >> 28) & 0xF;
        let e9 = (self.edges >> 36) & 0xF;

        let (ne1, ne5, ne7, ne9) = if PRIME {
            (e7, e1, e9, e5)
        } else {
            (e5, e9, e1, e7)
        };

        self.edges = (self.edges & !0x0000_00F0_F0F0_00F0)
            | (ne1 << 4)
            | (ne5 << 20)
            | (ne7 << 28)
            | (ne9 << 36);

        let c1 = (self.corners >> 8) & 0xFF;
        let c2 = (self.corners >> 16) & 0xFF;
        let c5 = (self.corners >> 40) & 0xFF;
        let c6 = (self.corners >> 48) & 0xFF;

        let (nc1, nc2, nc5, nc6) = if PRIME {
            (c2, c6, c1, c5)
        } else {
            (c5, c1, c6, c2)
        };

        self.corners = (self.corners & !0x00FF_FF00_00FF_FF00)
            | (nc1 << 8)
            | (nc2 << 16)
            | (nc5 << 40)
            | (nc6 << 48);

        // Orientation (with flip)
        let eo1 = (self.edges >> 49) & 1;
        let eo5 = (self.edges >> 53) & 1;
        let eo7 = (self.edges >> 55) & 1;
        let eo9 = (self.edges >> 57) & 1;
        let (ne_eo1, ne_eo5, ne_eo7, ne_eo9) = if PRIME {
            (eo7 ^ 1, eo1 ^ 1, eo9 ^ 1, eo5 ^ 1)
        } else {
            (eo5 ^ 1, eo9 ^ 1, eo1 ^ 1, eo7 ^ 1)
        };
        self.edges = (self.edges & !0x02A2_0000_0000_0000)
            | (ne_eo1 << 49)
            | (ne_eo5 << 53)
            | (ne_eo7 << 55)
            | (ne_eo9 << 57);
        // TODO: Corners
    }

    pub fn is_solved(&self) -> bool {
        self.edges == 0x0FFF_BA98_7654_3210 && self.corners == 0x0706_0504_0302_0100
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_rotate_four_times() {
        let mut cube = Cube::new();
        for _ in 0..3 {
            cube.right::<false>();
            assert!(!cube.is_solved());
        }
        cube.right::<false>();
        assert!(cube.is_solved());

        for _ in 0..3 {
            cube.left::<false>();
            assert!(!cube.is_solved());
        }
        cube.left::<false>();
        assert!(cube.is_solved());

        for _ in 0..3 {
            cube.up::<false>();
            assert!(!cube.is_solved());
        }
        cube.up::<false>();
        assert!(cube.is_solved());

        for _ in 0..3 {
            cube.down::<false>();
            assert!(!cube.is_solved());
        }
        cube.down::<false>();
        assert!(cube.is_solved());

        for _ in 0..3 {
            cube.front::<false>();
            assert!(!cube.is_solved());
        }
        cube.front::<false>();
        assert!(cube.is_solved());

        for _ in 0..3 {
            cube.back::<false>();
            assert!(!cube.is_solved());
        }
        cube.back::<false>();
        assert!(cube.is_solved());
    }

    #[test]
    fn test_prime_moves() {
        let mut cube = Cube::new();
        cube.right::<false>();
        assert!(!cube.is_solved());
        cube.right::<true>();
        assert!(cube.is_solved());

        cube.left::<false>();
        assert!(!cube.is_solved());
        cube.left::<true>();
        assert!(cube.is_solved());

        cube.up::<false>();
        assert!(!cube.is_solved());
        cube.up::<true>();
        assert!(cube.is_solved());

        cube.down::<false>();
        assert!(!cube.is_solved());
        cube.down::<true>();
        assert!(cube.is_solved());

        cube.front::<false>();
        assert!(!cube.is_solved());
        cube.front::<true>();
        assert!(cube.is_solved());

        cube.back::<false>();
        assert!(!cube.is_solved());
        cube.back::<true>();
        assert!(cube.is_solved());
    }

    #[test]
    fn test_sexy_move() {
        let mut cube = Cube::new();
        for _ in 0..5 {
            cube.right::<false>();
            cube.up::<false>();
            cube.right::<true>();
            cube.up::<true>();
            assert!(!cube.is_solved());
        }
        cube.right::<false>();
        cube.up::<false>();
        cube.right::<true>();
        cube.up::<true>();
        assert!(cube.is_solved());
    }
}
