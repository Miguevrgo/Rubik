pub const ZOBRIST_TABLE: [u64; 480] = {
    let mut table = [0u64; 480];
    let mut state = 0x123456789abcdef0u64;
    let mut i = 0;
    while i < 480 {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        table[i] = state;
        i += 1;
    }
    table
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Orientations {
    pub edges: u16,
    pub corners: u16,
}

impl Orientations {
    #[inline]
    pub fn get_edge(&self, index: usize) -> u8 {
        ((self.edges >> index) & 1) as u8
    }

    #[inline]
    pub fn set_edge(&mut self, index: usize, val: u8) {
        self.edges = (self.edges & !(1 << index)) | ((val as u16 & 1) << index);
    }

    #[inline]
    pub fn get_corner(&self, index: usize) -> u8 {
        ((self.corners >> (index * 2)) & 3) as u8
    }

    #[inline]
    pub fn set_corner(&mut self, index: usize, val: u8) {
        let shift = index * 2;
        self.corners = (self.corners & !(3 << shift)) | ((val as u16 & 3) << shift);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    U,
    UPrime,
    U2,
    D,
    DPrime,
    D2,
    R,
    RPrime,
    R2,
    L,
    LPrime,
    L2,
    F,
    FPrime,
    F2,
    B,
    BPrime,
    B2,
}

impl Move {
    pub fn name(self) -> &'static str {
        match self {
            Move::U => "U",
            Move::UPrime => "U'",
            Move::U2 => "U2",
            Move::D => "D",
            Move::DPrime => "D'",
            Move::D2 => "D2",
            Move::R => "R",
            Move::RPrime => "R'",
            Move::R2 => "R2",
            Move::L => "L",
            Move::LPrime => "L'",
            Move::L2 => "L2",
            Move::F => "F",
            Move::FPrime => "F'",
            Move::F2 => "F2",
            Move::B => "B",
            Move::BPrime => "B'",
            Move::B2 => "B2",
        }
    }

    pub fn face(self) -> u8 {
        match self {
            Move::U | Move::UPrime | Move::U2 => 0,
            Move::D | Move::DPrime | Move::D2 => 1,
            Move::L | Move::LPrime | Move::L2 => 2,
            Move::R | Move::RPrime | Move::R2 => 3,
            Move::F | Move::FPrime | Move::F2 => 4,
            Move::B | Move::BPrime | Move::B2 => 5,
        }
    }
}

pub const ALL_MOVES: [Move; 18] = [
    Move::U,
    Move::UPrime,
    Move::U2,
    Move::D,
    Move::DPrime,
    Move::D2,
    Move::R,
    Move::RPrime,
    Move::R2,
    Move::L,
    Move::LPrime,
    Move::L2,
    Move::F,
    Move::FPrime,
    Move::F2,
    Move::B,
    Move::BPrime,
    Move::B2,
];

/// The 12 quarter-turn moves only (for scramble generation compatibility).
pub const QUARTER_MOVES: [Move; 12] = [
    Move::U,
    Move::UPrime,
    Move::D,
    Move::DPrime,
    Move::R,
    Move::RPrime,
    Move::L,
    Move::LPrime,
    Move::F,
    Move::FPrime,
    Move::B,
    Move::BPrime,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cube {
    pub edges: u64,
    pub corners: u64,
    pub orientations: Orientations,
}

impl Cube {
    #[inline]
    pub fn apply_move(&mut self, mv: Move) {
        match mv {
            Move::U => self.up::<false>(),
            Move::UPrime => self.up::<true>(),
            Move::U2 => {
                self.up::<false>();
                self.up::<false>();
            }
            Move::D => self.down::<false>(),
            Move::DPrime => self.down::<true>(),
            Move::D2 => {
                self.down::<false>();
                self.down::<false>();
            }
            Move::R => self.right::<false>(),
            Move::RPrime => self.right::<true>(),
            Move::R2 => {
                self.right::<false>();
                self.right::<false>();
            }
            Move::L => self.left::<false>(),
            Move::LPrime => self.left::<true>(),
            Move::L2 => {
                self.left::<false>();
                self.left::<false>();
            }
            Move::F => self.front::<false>(),
            Move::FPrime => self.front::<true>(),
            Move::F2 => {
                self.front::<false>();
                self.front::<false>();
            }
            Move::B => self.back::<false>(),
            Move::BPrime => self.back::<true>(),
            Move::B2 => {
                self.back::<false>();
                self.back::<false>();
            }
        }
    }

    pub fn new() -> Self {
        Self {
            edges: 0xBA98_7654_3210,
            corners: 0x0706_0504_0302_0100,
            orientations: Orientations {
                edges: 0,
                corners: 0,
            },
        }
    }

    #[inline]
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

        let e0 = self.orientations.get_edge(0);
        let e1 = self.orientations.get_edge(1);
        let e2 = self.orientations.get_edge(2);
        let e3 = self.orientations.get_edge(3);
        let (ne0, ne1, ne2, ne3) = if PRIME {
            (e3, e0, e1, e2)
        } else {
            (e1, e2, e3, e0)
        };
        self.orientations.set_edge(0, ne0);
        self.orientations.set_edge(1, ne1);
        self.orientations.set_edge(2, ne2);
        self.orientations.set_edge(3, ne3);

        let c0 = self.orientations.get_corner(0);
        let c1 = self.orientations.get_corner(1);
        let c2 = self.orientations.get_corner(2);
        let c3 = self.orientations.get_corner(3);
        let (nc0, nc1, nc2, nc3) = if PRIME {
            (c3, c0, c1, c2)
        } else {
            (c1, c2, c3, c0)
        };
        self.orientations.set_corner(0, nc0);
        self.orientations.set_corner(1, nc1);
        self.orientations.set_corner(2, nc2);
        self.orientations.set_corner(3, nc3);
    }

    #[inline]
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

        let e8 = self.orientations.get_edge(8);
        let e9 = self.orientations.get_edge(9);
        let e10 = self.orientations.get_edge(10);
        let e11 = self.orientations.get_edge(11);
        let (ne8, ne9, ne10, ne11) = if PRIME {
            (e9, e10, e11, e8)
        } else {
            (e11, e8, e9, e10)
        };
        self.orientations.set_edge(8, ne8);
        self.orientations.set_edge(9, ne9);
        self.orientations.set_edge(10, ne10);
        self.orientations.set_edge(11, ne11);

        let c4 = self.orientations.get_corner(4);
        let c5 = self.orientations.get_corner(5);
        let c6 = self.orientations.get_corner(6);
        let c7 = self.orientations.get_corner(7);
        let (nc4, nc5, nc6, nc7) = if PRIME {
            (c5, c6, c7, c4)
        } else {
            (c7, c4, c5, c6)
        };
        self.orientations.set_corner(4, nc4);
        self.orientations.set_corner(5, nc5);
        self.orientations.set_corner(6, nc6);
        self.orientations.set_corner(7, nc7);
    }

    #[inline]
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

        let e0_ori = self.orientations.get_edge(0);
        let e4_ori = self.orientations.get_edge(4);
        let e5_ori = self.orientations.get_edge(5);
        let e8_ori = self.orientations.get_edge(8);
        let (ne0_ori, ne4_ori, ne5_ori, ne8_ori) = if PRIME {
            (e5_ori, e0_ori, e8_ori, e4_ori)
        } else {
            (e4_ori, e8_ori, e0_ori, e5_ori)
        };
        self.orientations.set_edge(0, ne0_ori);
        self.orientations.set_edge(4, ne4_ori);
        self.orientations.set_edge(5, ne5_ori);
        self.orientations.set_edge(8, ne8_ori);

        let c0_ori = self.orientations.get_corner(0);
        let c1_ori = self.orientations.get_corner(1);
        let c4_ori = self.orientations.get_corner(4);
        let c5_ori = self.orientations.get_corner(5);
        let (nc0_ori, nc1_ori, nc4_ori, nc5_ori) = if PRIME {
            (
                (c1_ori + 1) % 3,
                (c5_ori + 2) % 3,
                (c0_ori + 2) % 3,
                (c4_ori + 1) % 3,
            )
        } else {
            (
                (c4_ori + 1) % 3,
                (c0_ori + 2) % 3,
                (c5_ori + 2) % 3,
                (c1_ori + 1) % 3,
            )
        };
        self.orientations.set_corner(0, nc0_ori);
        self.orientations.set_corner(1, nc1_ori);
        self.orientations.set_corner(4, nc4_ori);
        self.orientations.set_corner(5, nc5_ori);
    }

    #[inline]
    pub fn left<const PRIME: bool>(&mut self) {
        let e2 = (self.edges >> 8) & 0xF;
        let e6 = (self.edges >> 24) & 0xF;
        let e7 = (self.edges >> 28) & 0xF;
        let e10 = (self.edges >> 40) & 0xF;
        let (ne2, ne6, ne7, ne10) = if PRIME {
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

        let e2_ori = self.orientations.get_edge(2);
        let e6_ori = self.orientations.get_edge(6);
        let e7_ori = self.orientations.get_edge(7);
        let e10_ori = self.orientations.get_edge(10);
        let (ne2_ori, ne6_ori, ne7_ori, ne10_ori) = if PRIME {
            (e7_ori, e2_ori, e10_ori, e6_ori)
        } else {
            (e6_ori, e10_ori, e2_ori, e7_ori)
        };
        self.orientations.set_edge(2, ne2_ori);
        self.orientations.set_edge(6, ne6_ori);
        self.orientations.set_edge(7, ne7_ori);
        self.orientations.set_edge(10, ne10_ori);

        let c2_ori = self.orientations.get_corner(2);
        let c3_ori = self.orientations.get_corner(3);
        let c6_ori = self.orientations.get_corner(6);
        let c7_ori = self.orientations.get_corner(7);
        let (nc2_ori, nc3_ori, nc6_ori, nc7_ori) = if PRIME {
            (
                (c3_ori + 1) % 3,
                (c7_ori + 2) % 3,
                (c2_ori + 2) % 3,
                (c6_ori + 1) % 3,
            )
        } else {
            (
                (c6_ori + 1) % 3,
                (c2_ori + 2) % 3,
                (c7_ori + 2) % 3,
                (c3_ori + 1) % 3,
            )
        };
        self.orientations.set_corner(2, nc2_ori);
        self.orientations.set_corner(3, nc3_ori);
        self.orientations.set_corner(6, nc6_ori);
        self.orientations.set_corner(7, nc7_ori);
    }

    #[inline]
    pub fn front<const PRIME: bool>(&mut self) {
        let e3 = (self.edges >> 12) & 0xF;
        let e4 = (self.edges >> 16) & 0xF;
        let e7 = (self.edges >> 28) & 0xF;
        let e11 = (self.edges >> 44) & 0xF;
        let (ne3, ne4, ne7, ne11) = if PRIME {
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
        let (nc0, nc3, nc4, nc7) = if PRIME {
            (c4, c0, c7, c3)
        } else {
            (c3, c7, c0, c4)
        };
        self.corners =
            (self.corners & !0xFF00_00FF_FF00_00FF) | nc0 | (nc3 << 24) | (nc4 << 32) | (nc7 << 56);

        let e3_ori = self.orientations.get_edge(3);
        let e4_ori = self.orientations.get_edge(4);
        let e7_ori = self.orientations.get_edge(7);
        let e11_ori = self.orientations.get_edge(11);
        let (ne3_ori, ne4_ori, ne7_ori, ne11_ori) = if PRIME {
            (1 - e4_ori, 1 - e11_ori, 1 - e3_ori, 1 - e7_ori)
        } else {
            (1 - e7_ori, 1 - e3_ori, 1 - e11_ori, 1 - e4_ori)
        };
        self.orientations.set_edge(3, ne3_ori);
        self.orientations.set_edge(4, ne4_ori);
        self.orientations.set_edge(7, ne7_ori);
        self.orientations.set_edge(11, ne11_ori);

        let c0_ori = self.orientations.get_corner(0);
        let c3_ori = self.orientations.get_corner(3);
        let c4_ori = self.orientations.get_corner(4);
        let c7_ori = self.orientations.get_corner(7);
        let (nc0_ori, nc3_ori, nc4_ori, nc7_ori) = if PRIME {
            (
                (c4_ori + 2) % 3,
                (c0_ori + 1) % 3,
                (c7_ori + 1) % 3,
                (c3_ori + 2) % 3,
            )
        } else {
            (
                (c3_ori + 2) % 3,
                (c7_ori + 1) % 3,
                (c0_ori + 1) % 3,
                (c4_ori + 2) % 3,
            )
        };
        self.orientations.set_corner(0, nc0_ori);
        self.orientations.set_corner(3, nc3_ori);
        self.orientations.set_corner(4, nc4_ori);
        self.orientations.set_corner(7, nc7_ori);
    }

    #[inline]
    pub fn back<const PRIME: bool>(&mut self) {
        let e1 = (self.edges >> 4) & 0xF;
        let e5 = (self.edges >> 20) & 0xF;
        let e6 = (self.edges >> 24) & 0xF;
        let e9 = (self.edges >> 36) & 0xF;
        let (ne1, ne5, ne6, ne9) = if PRIME {
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

        let e1_ori = self.orientations.get_edge(1);
        let e5_ori = self.orientations.get_edge(5);
        let e6_ori = self.orientations.get_edge(6);
        let e9_ori = self.orientations.get_edge(9);
        let (ne1_ori, ne5_ori, ne6_ori, ne9_ori) = if PRIME {
            (1 - e6_ori, 1 - e1_ori, 1 - e9_ori, 1 - e5_ori)
        } else {
            (1 - e5_ori, 1 - e9_ori, 1 - e1_ori, 1 - e6_ori)
        };
        self.orientations.set_edge(1, ne1_ori);
        self.orientations.set_edge(5, ne5_ori);
        self.orientations.set_edge(6, ne6_ori);
        self.orientations.set_edge(9, ne9_ori);

        let c1_ori = self.orientations.get_corner(1);
        let c2_ori = self.orientations.get_corner(2);
        let c5_ori = self.orientations.get_corner(5);
        let c6_ori = self.orientations.get_corner(6);
        let (nc1_ori, nc2_ori, nc5_ori, nc6_ori) = if PRIME {
            (
                (c2_ori + 1) % 3,
                (c6_ori + 2) % 3,
                (c1_ori + 2) % 3,
                (c5_ori + 1) % 3,
            )
        } else {
            (
                (c5_ori + 1) % 3,
                (c1_ori + 2) % 3,
                (c6_ori + 2) % 3,
                (c2_ori + 1) % 3,
            )
        };
        self.orientations.set_corner(1, nc1_ori);
        self.orientations.set_corner(2, nc2_ori);
        self.orientations.set_corner(5, nc5_ori);
        self.orientations.set_corner(6, nc6_ori);
    }

    pub fn is_solved(&self) -> bool {
        self.edges == 0xBA98_7654_3210
            && self.corners == 0x0706_0504_0302_0100
            && self.orientations.edges == 0
            && self.orientations.corners == 0
    }

    #[inline]
    pub fn to_features(self) -> [usize; 20] {
        let mut active_indices = [0; 20];
        for pos in 0..12 {
            let piece_id = ((self.edges >> (pos * 4)) & 0xF) as usize;
            let orientation = self.orientations.get_edge(pos) as usize;
            active_indices[pos] = pos * 24 + piece_id * 2 + orientation;
        }
        for pos in 0..8 {
            let piece_id = ((self.corners >> (pos * 8)) & 0xFF) as usize;
            let orientation = self.orientations.get_corner(pos) as usize;
            active_indices[12 + pos] = 288 + (pos * 24 + piece_id * 3 + orientation);
        }
        active_indices
    }

    #[inline]
    pub fn zobrist_hash(&self) -> u64 {
        let features = self.to_features();
        let mut hash = 0u64;
        for &feat in &features {
            hash ^= ZOBRIST_TABLE[feat];
        }
        hash
    }

    /// Encode corner permutation as a Lehmer code index in 0..40320 (8!).
    #[inline]
    pub fn corner_perm_index(&self) -> u32 {
        let mut perm = [0u8; 8];
        for i in 0..8 {
            perm[i] = ((self.corners >> (i * 8)) & 0xFF) as u8;
        }
        let mut idx = 0u32;
        for i in 0..8 {
            let mut count = 0u32;
            for j in (i + 1)..8 {
                if perm[j] < perm[i] {
                    count += 1;
                }
            }
            idx = idx * (8 - i as u32) + count;
        }
        idx
    }

    /// Encode corner orientations as a mixed-radix number in 0..2187 (3^7).
    /// The 8th orientation is determined by the other 7 (sum mod 3 = 0).
    #[inline]
    pub fn corner_ori_index(&self) -> u32 {
        let mut idx = 0u32;
        for i in 0..7 {
            idx = idx * 3 + self.orientations.get_corner(i) as u32;
        }
        idx
    }

    /// Combined corner index for pattern database: perm * 2187 + ori.
    /// Range: 0..88_179_840.
    #[inline]
    pub fn corner_index(&self) -> u32 {
        self.corner_perm_index() * 2187 + self.corner_ori_index()
    }

    /// Encoding the positions and orientations of the first 6 edges (pieces 0..5).
    /// Range: 0..42_577_920 (P(12, 6) * 2^6).
    #[inline]
    pub fn edge6_index(&self) -> usize {
        let mut pos = [0u8; 6];
        let mut ori = [0u8; 6];
        let mut found = 0;
        for p in 0..12 {
            let piece = ((self.edges >> (p * 4)) & 0xF) as u8;
            if piece < 6 {
                pos[piece as usize] = p as u8;
                ori[piece as usize] = self.orientations.get_edge(p);
                found += 1;
                if found == 6 {
                    break;
                }
            }
        }

        let mut perm_idx = 0u32;
        for i in 0..6 {
            let mut count = 0u32;
            for val in 0..pos[i] {
                let mut already_used = false;
                for j in 0..i {
                    if pos[j] == val {
                        already_used = true;
                        break;
                    }
                }
                if !already_used {
                    count += 1;
                }
            }
            perm_idx = perm_idx * (12 - i as u32) + count;
        }

        let mut ori_idx = 0u32;
        for i in 0..6 {
            ori_idx = (ori_idx << 1) | ori[i] as u32;
        }

        (perm_idx * 64 + ori_idx) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_solved_on_new() {
        let cube = Cube::new();
        assert!(cube.is_solved());
    }

    #[test]
    fn test_u_reversibility() {
        let mut cube = Cube::new();
        cube.up::<false>();
        assert!(!cube.is_solved());
        cube.up::<true>();
        assert!(cube.is_solved());
    }

    #[test]
    fn test_d_reversibility() {
        let mut cube = Cube::new();
        cube.down::<false>();
        assert!(!cube.is_solved());
        cube.down::<true>();
        assert!(cube.is_solved());
    }

    #[test]
    fn test_r_reversibility() {
        let mut cube = Cube::new();
        cube.right::<false>();
        assert!(!cube.is_solved());
        cube.right::<true>();
        assert!(cube.is_solved());
    }

    #[test]
    fn test_l_reversibility() {
        let mut cube = Cube::new();
        cube.left::<false>();
        assert!(!cube.is_solved());
        cube.left::<true>();
        assert!(cube.is_solved());
    }

    #[test]
    fn test_f_reversibility() {
        let mut cube = Cube::new();
        cube.front::<false>();
        assert!(!cube.is_solved());
        cube.front::<true>();
        assert!(cube.is_solved());
    }

    #[test]
    fn test_b_reversibility() {
        let mut cube = Cube::new();
        cube.back::<false>();
        assert!(!cube.is_solved());
        cube.back::<true>();
        assert!(cube.is_solved());
    }

    #[test]
    fn test_sexy_move_period() {
        let mut cube = Cube::new();
        for _ in 0..6 {
            cube.right::<false>();
            cube.up::<false>();
            cube.right::<true>();
            cube.up::<true>();
        }
        assert!(cube.is_solved());
    }

    #[test]
    fn test_scramble_and_inverse() {
        let mut cube = Cube::new();
        cube.up::<false>();
        cube.right::<false>();
        cube.front::<false>();
        cube.left::<false>();
        cube.back::<false>();
        cube.down::<false>();
        assert!(!cube.is_solved());
        cube.down::<true>();
        cube.back::<true>();
        cube.left::<true>();
        cube.front::<true>();
        cube.right::<true>();
        cube.up::<true>();
        assert!(cube.is_solved());
    }

    #[test]
    fn test_to_features_lengths_and_changes() {
        let mut cube = Cube::new();
        let initial_features = cube.to_features();
        assert_eq!(initial_features.len(), 20);
        for pos in 0..12 {
            assert_eq!(initial_features[pos], pos * 26);
        }
        for pos in 0..8 {
            assert_eq!(initial_features[12 + pos], 288 + pos * 27);
        }
        cube.up::<false>();
        let scramble_features = cube.to_features();
        assert_eq!(scramble_features.len(), 20);
        let mut identical_count = 0;
        for i in 0..20 {
            if initial_features.contains(&scramble_features[i]) {
                identical_count += 1;
            }
        }
        assert_eq!(identical_count, 12);
    }
}
