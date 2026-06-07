use crate::cube::{ALL_MOVES, Cube};
use serde::Deserialize;

#[derive(Deserialize)]
struct HeuristicModelJson {
    w1: Vec<Vec<f32>>,
    b1: Vec<f32>,
    w2: Vec<Vec<f32>>,
    b2: Vec<f32>,
    w3: Vec<f32>,
    b3: f32,
}

pub struct HeuristicModel {
    pub w1: Vec<f32>,
    pub b1: Vec<f32>,
    pub w2: Vec<f32>,
    pub b2: Vec<f32>,
    pub w3: Vec<f32>,
    pub b3: f32,
}

#[target_feature(enable = "avx512f")]
unsafe fn sum_row_avx512(acc: *mut f32, w: *const f32) {
    unsafe {
        for i in 0..16 {
            let pa = acc.add(i * 16);
            let pw = w.add(i * 16);
            std::arch::x86_64::_mm512_storeu_ps(
                pa,
                std::arch::x86_64::_mm512_add_ps(
                    std::arch::x86_64::_mm512_loadu_ps(pa),
                    std::arch::x86_64::_mm512_loadu_ps(pw),
                ),
            );
        }
    }
}

#[target_feature(enable = "avx512f")]
unsafe fn fmadd_row_avx512(acc: *mut f32, w: *const f32, factor: f32) {
    unsafe {
        let vf = std::arch::x86_64::_mm512_set1_ps(factor);
        for i in 0..8 {
            let pa = acc.add(i * 16);
            let pw = w.add(i * 16);
            std::arch::x86_64::_mm512_storeu_ps(
                pa,
                std::arch::x86_64::_mm512_fmadd_ps(
                    vf,
                    std::arch::x86_64::_mm512_loadu_ps(pw),
                    std::arch::x86_64::_mm512_loadu_ps(pa),
                ),
            );
        }
    }
}

#[target_feature(enable = "avx512f")]
unsafe fn relu_avx512<const N: usize>(acc: *mut f32) {
    unsafe {
        let z = std::arch::x86_64::_mm512_setzero_ps();
        for i in 0..N {
            let p = acc.add(i * 16);
            std::arch::x86_64::_mm512_storeu_ps(
                p,
                std::arch::x86_64::_mm512_max_ps(std::arch::x86_64::_mm512_loadu_ps(p), z),
            );
        }
    }
}

#[target_feature(enable = "avx512f")]
unsafe fn dot128_avx512(a: *const f32, b: *const f32) -> f32 {
    unsafe {
        let mut s = std::arch::x86_64::_mm512_setzero_ps();
        for i in 0..8 {
            s = std::arch::x86_64::_mm512_fmadd_ps(
                std::arch::x86_64::_mm512_loadu_ps(a.add(i * 16)),
                std::arch::x86_64::_mm512_loadu_ps(b.add(i * 16)),
                s,
            );
        }
        std::arch::x86_64::_mm512_reduce_add_ps(s)
    }
}

impl HeuristicModel {
    pub fn load() -> Result<Self, String> {
        let data = std::fs::read_to_string("nnue_model_weights.json")
            .map_err(|e| format!("Could not read nnue_model_weights.json: {e}"))?;
        let j: HeuristicModelJson = serde_json::from_str(&data)
            .map_err(|e| format!("Error deserializing nnue_model_weights.json: {e}"))?;
        Ok(Self {
            w1: j.w1.into_iter().flatten().collect(),
            b1: j.b1,
            w2: j.w2.into_iter().flatten().collect(),
            b2: j.b2,
            w3: j.w3,
            b3: j.b3,
        })
    }

    #[inline]
    pub fn predict(&self, feats: &[usize]) -> f32 {
        let mut h1 = [0.0f32; 256];
        h1.copy_from_slice(&self.b1);
        unsafe {
            let p1 = h1.as_mut_ptr();
            let w = self.w1.as_ptr();
            for &i in feats {
                if i < 480 {
                    sum_row_avx512(p1, w.add(i * 256));
                }
            }
            relu_avx512::<16>(p1);
        }
        let mut h2 = [0.0f32; 128];
        h2.copy_from_slice(&self.b2);
        unsafe {
            let p1 = h1.as_ptr();
            let p2 = h2.as_mut_ptr();
            let w = self.w2.as_ptr();
            for i in 0..256 {
                let v = *p1.add(i);
                if v > 0.0 {
                    fmadd_row_avx512(p2, w.add(i * 128), v);
                }
            }
            relu_avx512::<8>(p2);
        }
        (unsafe { dot128_avx512(h2.as_ptr(), self.w3.as_ptr()) }) + self.b3
    }
}

const CORNER_PDB_SIZE: usize = 88_179_840;

pub struct CornerPdb {
    data: Vec<u8>,
}

impl CornerPdb {
    pub fn precompute() -> Self {
        let path = std::path::Path::new("corner_pdb.bin");
        if path.exists() {
            eprintln!("[Corner PDB] Loading from corner_pdb.bin...");
            if let Ok(data) = std::fs::read(path) {
                if data.len() == CORNER_PDB_SIZE.div_ceil(2) {
                    return CornerPdb { data };
                }
            }
            eprintln!("[Corner PDB] Failed to load or invalid size, recomputing...");
        }

        let packed_len = CORNER_PDB_SIZE.div_ceil(2);
        let mut data = vec![0xFFu8; packed_len];

        let solved = Cube::new();
        let si = solved.corner_index() as usize;

        Self::set_dist(&mut data, si, 0);

        let mut current_level = Vec::new();
        let mut next_level = Vec::new();
        current_level.push((solved.corners, solved.orientations.corners));

        let mut count = 1usize;
        let mut d = 0u8;

        while !current_level.is_empty() && d < 11 {
            for &(corners, corner_ori) in &current_level {
                for &m in &ALL_MOVES {
                    let mut c = Cube {
                        edges: 0xBA98_7654_3210,
                        corners,
                        orientations: crate::cube::Orientations {
                            edges: 0,
                            corners: corner_ori,
                        },
                    };
                    c.apply_move(m);
                    let idx = c.corner_index() as usize;

                    if Self::get_dist(&data, idx) == 0xF {
                        Self::set_dist(&mut data, idx, d + 1);
                        count += 1;
                        next_level.push((c.corners, c.orientations.corners));
                    }
                }
            }
            current_level.clear();
            std::mem::swap(&mut current_level, &mut next_level);
            d += 1;
            eprintln!(
                "[Corner PDB] Depth {d} complete, level size: {}",
                current_level.len()
            );
        }

        eprintln!("[Corner PDB] Filled {count}/{CORNER_PDB_SIZE} entries");
        let pdb = CornerPdb { data };
        if let Err(e) = std::fs::write(path, &pdb.data) {
            eprintln!("[Corner PDB] Failed to save to corner_pdb.bin: {e}");
        }
        pdb
    }

    #[inline]
    fn get_dist(data: &[u8], idx: usize) -> u8 {
        let byte = data[idx >> 1];
        if idx & 1 == 0 { byte & 0xF } else { byte >> 4 }
    }

    #[inline]
    fn set_dist(data: &mut [u8], idx: usize, val: u8) {
        let bi = idx >> 1;
        if idx & 1 == 0 {
            data[bi] = (data[bi] & 0xF0) | (val & 0xF);
        } else {
            data[bi] = (data[bi] & 0x0F) | ((val & 0xF) << 4);
        }
    }

    #[inline]
    pub fn lookup(&self, cube: &Cube) -> u8 {
        Self::get_dist(&self.data, cube.corner_index() as usize)
    }
}

const EDGE6_PDB_SIZE: usize = 42_577_920;

pub struct Edge6Pdb {
    data: Vec<u8>,
}

impl Edge6Pdb {
    pub fn precompute() -> Self {
        let path = std::path::Path::new("edge6_pdb.bin");
        if path.exists() {
            eprintln!("[Edge6 PDB] Loading from edge6_pdb.bin...");
            if let Ok(data) = std::fs::read(path) {
                if data.len() == EDGE6_PDB_SIZE.div_ceil(2) {
                    return Edge6Pdb { data };
                }
            }
            eprintln!("[Edge6 PDB] Failed to load or invalid size, recomputing...");
        }

        let packed_len = EDGE6_PDB_SIZE.div_ceil(2);
        let mut data = vec![0xFFu8; packed_len];

        let solved = Cube::new();
        let si = solved.edge6_index();

        Self::set_dist(&mut data, si, 0);

        let mut current_level = Vec::new();
        let mut next_level = Vec::new();
        current_level.push(solved);

        let mut count = 1usize;
        let mut d = 0u8;

        while !current_level.is_empty() {
            for &cube in &current_level {
                for &m in &ALL_MOVES {
                    let mut nc = cube;
                    nc.apply_move(m);
                    let idx = nc.edge6_index();

                    if Self::get_dist(&data, idx) == 0xF {
                        Self::set_dist(&mut data, idx, d + 1);
                        count += 1;
                        next_level.push(nc);
                    }
                }
            }
            current_level.clear();
            std::mem::swap(&mut current_level, &mut next_level);
            d += 1;
            eprintln!(
                "[Edge6 PDB] Depth {d} complete, level size: {}, total filled: {count}",
                current_level.len()
            );
        }

        eprintln!("[Edge6 PDB] Filled {count}/{EDGE6_PDB_SIZE} entries");
        let pdb = Edge6Pdb { data };
        if let Err(e) = std::fs::write(path, &pdb.data) {
            eprintln!("[Edge6 PDB] Failed to save to edge6_pdb.bin: {e}");
        }
        pdb
    }

    #[inline]
    fn get_dist(data: &[u8], idx: usize) -> u8 {
        let byte = data[idx >> 1];
        if idx & 1 == 0 { byte & 0xF } else { byte >> 4 }
    }

    #[inline]
    fn set_dist(data: &mut [u8], idx: usize, val: u8) {
        let bi = idx >> 1;
        if idx & 1 == 0 {
            data[bi] = (data[bi] & 0xF0) | (val & 0xF);
        } else {
            data[bi] = (data[bi] & 0x0F) | ((val & 0xF) << 4);
        }
    }

    #[inline]
    pub fn lookup(&self, cube: &Cube) -> u8 {
        Self::get_dist(&self.data, cube.edge6_index())
    }
}
