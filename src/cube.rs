use rand::RngExt;

const ADD1: [u64; 16] = [1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const ADD2: [u64; 16] = [2, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

/// There are 12 edges, numbered anticlockwise starting from 6 always
/// This numeration fits into 4 bits which are also clustered inside
/// an u64 as we need 4 * 12 = 48 bits, then we use next 12 bits for
/// the orientation
///
/// There are 8 corners, numbered from top to bottom anticlockwise.
/// We use 32 bits for the corner identities (8 corners * 4 bits) inside the
/// bottom 32 bits of a u64, and the upper 32 bits for the corner orientations
/// (8 corners * 4 bits), totaling 64 bits.
#[derive(Copy, Clone)]
pub struct Cube {
    edges: u64,
    corners: u64,
}

impl Cube {
    pub fn new() -> Self {
        Self {
            edges: 0x0FFF_BA98_7654_3210,
            corners: 0x0000_0000_7654_3210,
        }
    }

    /// Eeach possible move (clockwise or anticlockwise)
    fn up<const PRIME: bool>(&mut self) {
        let edges_face = (self.edges & 0xFFFF) as u16;
        let rotated_edges = if PRIME {
            edges_face.rotate_left(4)
        } else {
            edges_face.rotate_right(4)
        };
        self.edges = (self.edges & !0xFFFF) | (rotated_edges as u64);

        let corners_face = (self.corners & 0xFFFF) as u16;
        let rotated_corners = if PRIME {
            corners_face.rotate_left(4)
        } else {
            corners_face.rotate_right(4)
        };
        self.corners = (self.corners & !0xFFFF) | (rotated_corners as u64);

        // Orientation Edges
        let eo_bits = ((self.edges >> 48) & 0xF) as u8;
        let rotated_eo = if PRIME {
            ((eo_bits << 1) & 0xF) | (eo_bits >> 3)
        } else {
            (eo_bits >> 1) | ((eo_bits & 1) << 3)
        };
        self.edges = (self.edges & !0x000F_0000_0000_0000) | ((rotated_eo as u64) << 48);

        // Orientation Corners
        let co_face = ((self.corners >> 32) & 0xFFFF) as u16;
        let rotated_co = if PRIME {
            co_face.rotate_left(4)
        } else {
            co_face.rotate_right(4)
        };
        self.corners = (self.corners & !0x0000_FFFF_0000_0000) | ((rotated_co as u64) << 32);
    }

    fn down<const PRIME: bool>(&mut self) {
        let edges_face = (self.edges >> 32) as u16;
        let rotated_edges = if PRIME {
            edges_face.rotate_right(4)
        } else {
            edges_face.rotate_left(4)
        };
        self.edges = (self.edges & !0xFFFF_0000_0000) | ((rotated_edges as u64) << 32);

        let corners_face = ((self.corners >> 16) & 0xFFFF) as u16;
        let rotated_corners = if PRIME {
            corners_face.rotate_right(4)
        } else {
            corners_face.rotate_left(4)
        };
        self.corners = (self.corners & !0xFFFF_0000) | ((rotated_corners as u64) << 16);

        // Orientation Edges
        let eo_bits = ((self.edges >> 56) & 0xF) as u8;
        let rotated_eo = if PRIME {
            (eo_bits >> 1) | ((eo_bits & 1) << 3)
        } else {
            ((eo_bits << 1) & 0xF) | (eo_bits >> 3)
        };
        self.edges = (self.edges & !0x0F00_0000_0000_0000) | ((rotated_eo as u64) << 56);

        // Orientation Corners
        let co_face = ((self.corners >> 48) & 0xFFFF) as u16;
        let rotated_co = if PRIME {
            co_face.rotate_right(4)
        } else {
            co_face.rotate_left(4)
        };
        self.corners = (self.corners & !0xFFFF_0000_0000_0000) | ((rotated_co as u64) << 48);
    }

    fn right<const PRIME: bool>(&mut self) {
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

        let c0 = self.corners & 0xF;
        let c1 = (self.corners >> 4) & 0xF;
        let c4 = (self.corners >> 16) & 0xF;
        let c5 = (self.corners >> 20) & 0xF;

        let (nc0, nc1, nc4, nc5) = if PRIME {
            (c1, c5, c0, c4)
        } else {
            (c4, c0, c5, c1)
        };

        let co0 = (self.corners >> 32) & 0xF;
        let co1 = (self.corners >> 36) & 0xF;
        let co4 = (self.corners >> 48) & 0xF;
        let co5 = (self.corners >> 52) & 0xF;

        let (nco0, nco1, nco4, nco5) = if PRIME {
            (
                ADD1[co1 as usize],
                ADD2[co5 as usize],
                ADD2[co0 as usize],
                ADD1[co4 as usize],
            )
        } else {
            (
                ADD1[co4 as usize],
                ADD2[co0 as usize],
                ADD2[co5 as usize],
                ADD1[co1 as usize],
            )
        };

        self.corners = (self.corners & !0x00FF_00FF_00FF_00FF)
            | nc0
            | (nc1 << 4)
            | (nc4 << 16)
            | (nc5 << 20)
            | (nco0 << 32)
            | (nco1 << 36)
            | (nco4 << 48)
            | (nco5 << 52);

        // Orientation Edges
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
    }

    fn left<const PRIME: bool>(&mut self) {
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

        let c2 = (self.corners >> 8) & 0xF;
        let c3 = (self.corners >> 12) & 0xF;
        let c6 = (self.corners >> 24) & 0xF;
        let c7 = (self.corners >> 28) & 0xF;

        let (nc2, nc3, nc6, nc7) = if PRIME {
            (c3, c7, c2, c6)
        } else {
            (c6, c2, c7, c3)
        };

        let co2 = (self.corners >> 40) & 0xF;
        let co3 = (self.corners >> 44) & 0xF;
        let co6 = (self.corners >> 56) & 0xF;
        let co7 = (self.corners >> 60) & 0xF;

        let (nco2, nco3, nco6, nco7) = if PRIME {
            (
                ADD1[co3 as usize],
                ADD2[co7 as usize],
                ADD2[co2 as usize],
                ADD1[co6 as usize],
            )
        } else {
            (
                ADD1[co6 as usize],
                ADD2[co2 as usize],
                ADD2[co7 as usize],
                ADD1[co3 as usize],
            )
        };

        self.corners = (self.corners & !0xFF00_FF00_FF00_FF00)
            | (nc2 << 8)
            | (nc3 << 12)
            | (nc6 << 24)
            | (nc7 << 28)
            | (nco2 << 40)
            | (nco3 << 44)
            | (nco6 << 56)
            | (nco7 << 60);

        // Orientation Edges
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
    }

    fn front<const PRIME: bool>(&mut self) {
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

        let c0 = self.corners & 0xF;
        let c3 = (self.corners >> 12) & 0xF;
        let c4 = (self.corners >> 16) & 0xF;
        let c7 = (self.corners >> 28) & 0xF;

        let (nc0, nc3, nc4, nc7) = if PRIME {
            (c4, c0, c7, c3)
        } else {
            (c3, c7, c0, c4)
        };

        let co0 = (self.corners >> 32) & 0xF;
        let co3 = (self.corners >> 44) & 0xF;
        let co4 = (self.corners >> 48) & 0xF;
        let co7 = (self.corners >> 60) & 0xF;

        let (nco0, nco3, nco4, nco7) = if PRIME {
            (
                ADD2[co4 as usize],
                ADD1[co0 as usize],
                ADD1[co7 as usize],
                ADD2[co3 as usize],
            )
        } else {
            (
                ADD2[co3 as usize],
                ADD1[co7 as usize],
                ADD1[co0 as usize],
                ADD2[co4 as usize],
            )
        };

        self.corners = (self.corners & !0xF00F_F00F_F00F_F00F)
            | nc0
            | (nc3 << 12)
            | (nc4 << 16)
            | (nc7 << 28)
            | (nco0 << 32)
            | (nco3 << 44)
            | (nco4 << 48)
            | (nco7 << 60);

        // Orientation Edges (with flip)
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
    }

    fn back<const PRIME: bool>(&mut self) {
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

        let c1 = (self.corners >> 4) & 0xF;
        let c2 = (self.corners >> 8) & 0xF;
        let c5 = (self.corners >> 20) & 0xF;
        let c6 = (self.corners >> 24) & 0xF;

        let (nc1, nc2, nc5, nc6) = if PRIME {
            (c2, c6, c1, c5)
        } else {
            (c5, c1, c6, c2)
        };

        let co1 = (self.corners >> 36) & 0xF;
        let co2 = (self.corners >> 40) & 0xF;
        let co5 = (self.corners >> 52) & 0xF;
        let co6 = (self.corners >> 56) & 0xF;

        let (nco1, nco2, nco5, nco6) = if PRIME {
            (
                ADD1[co2 as usize],
                ADD2[co6 as usize],
                ADD2[co1 as usize],
                ADD1[co5 as usize],
            )
        } else {
            (
                ADD1[co5 as usize],
                ADD2[co1 as usize],
                ADD2[co6 as usize],
                ADD1[co2 as usize],
            )
        };

        self.corners = (self.corners & !0x0FF0_0FF0_0FF0_0FF0)
            | (nc1 << 4)
            | (nc2 << 8)
            | (nc5 << 20)
            | (nc6 << 24)
            | (nco1 << 36)
            | (nco2 << 40)
            | (nco5 << 52)
            | (nco6 << 56);

        // Orientation Edges (with flip)
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
    }

    pub fn is_solved(&self) -> bool {
        self.edges == 0x0FFF_BA98_7654_3210 && self.corners == 0x0000_0000_7654_3210
    }

    pub fn shuffle(&mut self, n: usize) -> Vec<Move> {
        let mut rng = rand::rng();
        let mut moves = Vec::with_capacity(n);
        for _ in 0..n {
            let mv = Move::ALL[rng.random_range(0..12)];
            self.apply_move(mv);
            moves.push(mv);
        }
        moves
    }

    fn get_corner_color(&self, pos: usize, facelet_type: usize) -> char {
        const CORNER_COLORS: [[char; 3]; 8] = [
            ['W', 'R', 'G'],
            ['W', 'B', 'R'],
            ['W', 'O', 'B'],
            ['W', 'G', 'O'],
            ['Y', 'G', 'R'],
            ['Y', 'R', 'B'],
            ['Y', 'B', 'O'],
            ['Y', 'O', 'G'],
        ];
        let id = ((self.corners >> (pos * 4)) & 0xF) as usize;
        let co = ((self.corners >> (32 + pos * 4)) & 0xF) as usize;
        let colors = CORNER_COLORS[id];
        match co {
            0 => colors[facelet_type],
            1 => {
                colors[if facelet_type == 2 {
                    0
                } else {
                    facelet_type + 1
                }]
            }
            _ => {
                colors[if facelet_type == 0 {
                    2
                } else {
                    facelet_type - 1
                }]
            }
        }
    }

    fn get_edge_color(&self, pos: usize, is_secondary: bool) -> char {
        const EDGE_COLORS: [[char; 2]; 12] = [
            ['W', 'R'],
            ['W', 'B'],
            ['W', 'O'],
            ['W', 'G'],
            ['G', 'R'],
            ['B', 'R'],
            ['G', 'O'],
            ['B', 'O'],
            ['Y', 'R'],
            ['Y', 'B'],
            ['Y', 'O'],
            ['Y', 'G'],
        ];
        let id = ((self.edges >> (pos * 4)) & 0xF) as usize;
        let eo = ((self.edges >> (48 + pos)) & 1) as usize;
        EDGE_COLORS[id][(eo == 0) as usize ^ is_secondary as usize]
    }

    fn get_color(&self, face: char, r: usize, c: usize) -> char {
        match (face, r, c) {
            (_, 1, 1) => match face {
                'U' => 'W',
                'D' => 'Y',
                'L' => 'O',
                'F' => 'G',
                'R' => 'R',
                'B' => 'B',
                _ => ' ',
            },
            ('U', 0, 0) => self.get_corner_color(2, 0),
            ('U', 0, 2) => self.get_corner_color(1, 0),
            ('U', 2, 0) => self.get_corner_color(3, 0),
            ('U', 2, 2) => self.get_corner_color(0, 0),

            ('D', 0, 0) => self.get_corner_color(7, 0),
            ('D', 0, 2) => self.get_corner_color(4, 0),
            ('D', 2, 0) => self.get_corner_color(6, 0),
            ('D', 2, 2) => self.get_corner_color(5, 0),

            ('L', 0, 0) => self.get_corner_color(2, 1),
            ('L', 0, 2) => self.get_corner_color(3, 2),
            ('L', 2, 0) => self.get_corner_color(6, 2),
            ('L', 2, 2) => self.get_corner_color(7, 1),

            ('F', 0, 0) => self.get_corner_color(3, 1),
            ('F', 0, 2) => self.get_corner_color(0, 2),
            ('F', 2, 0) => self.get_corner_color(7, 2),
            ('F', 2, 2) => self.get_corner_color(4, 1),

            ('R', 0, 0) => self.get_corner_color(0, 1),
            ('R', 0, 2) => self.get_corner_color(1, 2),
            ('R', 2, 0) => self.get_corner_color(4, 2),
            ('R', 2, 2) => self.get_corner_color(5, 1),

            ('B', 0, 0) => self.get_corner_color(1, 1),
            ('B', 0, 2) => self.get_corner_color(2, 2),
            ('B', 2, 0) => self.get_corner_color(5, 2),
            ('B', 2, 2) => self.get_corner_color(6, 1),

            ('U', 0, 1) => self.get_edge_color(1, false),
            ('U', 1, 0) => self.get_edge_color(2, false),
            ('U', 1, 2) => self.get_edge_color(0, false),
            ('U', 2, 1) => self.get_edge_color(3, false),

            ('D', 0, 1) => self.get_edge_color(11, false),
            ('D', 1, 0) => self.get_edge_color(10, false),
            ('D', 1, 2) => self.get_edge_color(8, false),
            ('D', 2, 1) => self.get_edge_color(9, false),

            ('L', 0, 1) => self.get_edge_color(2, true),
            ('L', 1, 0) => self.get_edge_color(7, true),
            ('L', 1, 2) => self.get_edge_color(6, true),
            ('L', 2, 1) => self.get_edge_color(10, true),

            ('F', 0, 1) => self.get_edge_color(3, true),
            ('F', 1, 0) => self.get_edge_color(6, false),
            ('F', 1, 2) => self.get_edge_color(4, false),
            ('F', 2, 1) => self.get_edge_color(11, true),

            ('R', 0, 1) => self.get_edge_color(0, true),
            ('R', 1, 0) => self.get_edge_color(4, true),
            ('R', 1, 2) => self.get_edge_color(5, true),
            ('R', 2, 1) => self.get_edge_color(8, true),

            ('B', 0, 1) => self.get_edge_color(1, true),
            ('B', 1, 0) => self.get_edge_color(5, false),
            ('B', 1, 2) => self.get_edge_color(7, false),
            ('B', 2, 1) => self.get_edge_color(9, true),

            _ => ' ',
        }
    }

    pub fn apply_move(&mut self, mv: Move) {
        match mv {
            Move::U => self.up::<false>(),
            Move::UPrime => self.up::<true>(),
            Move::D => self.down::<false>(),
            Move::DPrime => self.down::<true>(),
            Move::R => self.right::<false>(),
            Move::RPrime => self.right::<true>(),
            Move::L => self.left::<false>(),
            Move::LPrime => self.left::<true>(),
            Move::F => self.front::<false>(),
            Move::FPrime => self.front::<true>(),
            Move::B => self.back::<false>(),
            Move::BPrime => self.back::<true>(),
            Move::NULL => {}
        }
    }

    // TODO:Use NNUE
    #[allow(dead_code)]
    pub fn evaluate(&self) -> i32 {
        unimplemented!()
    }

    /// We use xorshift64star for the hashing
    /// https://en.wikipedia.org/wiki/Xorshift
    pub fn hash(&self) -> u64 {
        let mut x = self.edges ^ self.corners;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;

        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
}

impl std::fmt::Display for Cube {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const RED: &str = "\x1b[1;31;49m";
        const GREEN: &str = "\x1b[1;32;49m";
        const ORANGE: &str = "\x1b[1;38;5;202;49m";
        const YELLOW: &str = "\x1b[1;33;49m";
        const BLUE: &str = "\x1b[1;34;49m";
        const RESET: &str = "\x1b[1;39;49m";

        let color = |face: char, r: usize, c: usize| -> String {
            let val = self.get_color(face, r, c);
            let color_code = match val {
                'R' => RED,
                'G' => GREEN,
                'O' => ORANGE,
                'Y' => YELLOW,
                'B' => BLUE,
                _ => RESET,
            };
            format!("{}{}{}", color_code, val, RESET)
        };

        let get_row = |face: char, r: usize| -> String {
            format!(
                "[{} {} {}]",
                color(face, r, 0),
                color(face, r, 1),
                color(face, r, 2)
            )
        };

        for r in 0..3 {
            writeln!(f, "{}{}", " ".repeat(7), get_row('U', r))?;
        }

        for r in 0..3 {
            writeln!(
                f,
                "{}{}{}{}",
                get_row('L', r),
                get_row('F', r),
                get_row('R', r),
                get_row('B', r)
            )?;
        }

        for r in 0..3 {
            writeln!(f, "{}{}", " ".repeat(7), get_row('D', r))?;
        }

        Ok(())
    }
}

/// This way we have all the moves in "pairs" where
/// all the bits except the first one denote the type
/// while the last bit is 0 for normal or 1 for prime
#[repr(u8)]
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    U = 0,
    UPrime = 1,
    D = 2,
    DPrime = 3,
    R = 4,
    RPrime = 5,
    L = 6,
    LPrime = 7,
    F = 8,
    FPrime = 9,
    B = 10,
    BPrime = 11,
    NULL = 12,
}

impl Move {
    pub const ALL: [Self; 12] = [
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

    pub fn gen_moves(last_move: Move) -> impl Iterator<Item = Move> {
        Move::ALL
            .into_iter()
            .filter(move |&mv| mv as u8 != last_move as u8 ^ 1)
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
