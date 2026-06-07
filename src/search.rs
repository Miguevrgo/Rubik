#![allow(clippy::collapsible_if, clippy::map_entry, clippy::too_many_arguments)]

use crate::cube::{ALL_MOVES, Cube, Move};
use serde::Deserialize;
use std::collections::{HashMap, VecDeque};

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

const MAX_BFS_DEPTH: u8 = 6;

#[derive(Clone, Copy)]
pub struct SolvedEntry {
    pub hash: u64,
    pub dist: u8,
}

pub struct SolvedTable {
    pub entries: Vec<SolvedEntry>,
}

impl SolvedTable {
    pub fn precompute() -> Self {
        let mut dist = HashMap::new();
        let s = Cube::new();
        dist.insert(s.zobrist_hash(), 0u8);
        let mut q = VecDeque::new();
        q.push_back((s, 0u8));
        while let Some((c, d)) = q.pop_front() {
            if d >= MAX_BFS_DEPTH {
                continue;
            }
            for &m in &ALL_MOVES {
                let mut nc = c;
                nc.apply_move(m);
                let h = nc.zobrist_hash();
                if !dist.contains_key(&h) {
                    dist.insert(h, d + 1);
                    q.push_back((nc, d + 1));
                }
            }
        }
        eprintln!(
            "[BFS table] {} states at depth <= {MAX_BFS_DEPTH}",
            dist.len()
        );

        let size = 8_388_608;
        let mut entries = vec![
            SolvedEntry {
                hash: u64::MAX,
                dist: 0
            };
            size
        ];
        let mask = size - 1;
        for (&hash, &d) in &dist {
            let mut idx = (hash as usize) & mask;
            while entries[idx].hash != u64::MAX {
                idx = (idx + 1) & mask;
            }
            entries[idx] = SolvedEntry { hash, dist: d };
        }
        SolvedTable { entries }
    }

    #[inline]
    pub fn get_distance(&self, hash: u64) -> Option<u8> {
        let mask = self.entries.len() - 1;
        let mut idx = (hash as usize) & mask;
        loop {
            let entry = self.entries[idx];
            if entry.hash == hash {
                return Some(entry.dist);
            }
            if entry.hash == u64::MAX {
                return None;
            }
            idx = (idx + 1) & mask;
        }
    }
}

pub enum SearchStatus {
    Found,
    NotFound,
}

#[derive(Clone, Copy)]
pub struct TtEntry {
    pub hash: u64,
    pub depth: u8,
    pub value: u8,
    pub solve_id: u16,
}

const SOLVE_TIMEOUT_SECS: u64 = 10;
const TT_SIZE: usize = 16_777_216;

thread_local! {
    static TT: std::cell::RefCell<Vec<TtEntry>> = std::cell::RefCell::new(
        vec![TtEntry { hash: u64::MAX, depth: 0, value: 0, solve_id: 0 }; TT_SIZE]
    );
    static CURRENT_SOLVE_ID: std::cell::Cell<u16> = const { std::cell::Cell::new(0) };
}

pub struct IdaStarSolver {
    pub solved_table: SolvedTable,
    pub corner_pdb: CornerPdb,
    pub edge6_pdb: Edge6Pdb,
    pub heuristic: Option<HeuristicModel>,
}

impl IdaStarSolver {
    pub fn new() -> Self {
        let corner_pdb = CornerPdb::precompute();
        let edge6_pdb = Edge6Pdb::precompute();
        let solved_table = SolvedTable::precompute();
        let heuristic = HeuristicModel::load().ok();
        Self {
            solved_table,
            corner_pdb,
            edge6_pdb,
            heuristic,
        }
    }

    pub fn solve_from_table(&self, cube: &Cube) -> Option<Vec<Move>> {
        let mut path = Vec::new();
        if self.trace_table_path(cube, &mut path) {
            Some(path)
        } else {
            None
        }
    }

    fn trace_table_path(&self, cube: &Cube, path: &mut Vec<Move>) -> bool {
        let mut cur = *cube;
        loop {
            let h = cur.zobrist_hash();
            let d = match self.solved_table.get_distance(h) {
                Some(d) => d,
                None => return false,
            };
            if d == 0 {
                return true;
            }
            let mut ok = false;
            for &m in &ALL_MOVES {
                let mut nc = cur;
                nc.apply_move(m);
                if let Some(nd) = self.solved_table.get_distance(nc.zobrist_hash()) {
                    if nd == d - 1 {
                        path.push(m);
                        cur = nc;
                        ok = true;
                        break;
                    }
                }
            }
            if !ok {
                return false;
            }
        }
    }

    #[inline]
    fn heuristic(&self, cube: &Cube) -> u8 {
        let cpdb = self.corner_pdb.lookup(cube);
        let epdb = self.edge6_pdb.lookup(cube);
        let mut h = cpdb.max(epdb);
        if let Some(ref nnue) = self.heuristic {
            let raw = nnue.predict(&cube.to_features());
            let adj = (raw * 0.7).floor().max(0.0) as u8;
            h = h.max(adj);
        }
        h
    }

    pub fn solve(&self, cube: &Cube) -> Option<Vec<Move>> {
        if let Some(p) = self.solve_from_table(cube) {
            return Some(p);
        }
        let t0 = std::time::Instant::now();
        self.ida_star(cube, t0)
    }

    fn ida_star(&self, cube: &Cube, t0: std::time::Instant) -> Option<Vec<Move>> {
        let mut bound = self.heuristic(cube);
        let mut path = Vec::new();
        let mut path_hashes = Vec::with_capacity(32);

        thread_local! {
            static NODE_COUNTER: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
        }
        NODE_COUNTER.with(|c| c.set(0));

        let solve_id = CURRENT_SOLVE_ID.with(|id| {
            let mut val = id.get().wrapping_add(1);
            if val == 0 {
                TT.with(|tt| {
                    for entry in tt.borrow_mut().iter_mut() {
                        entry.hash = u64::MAX;
                        entry.solve_id = 0;
                    }
                });
                val = 1;
            }
            id.set(val);
            val
        });

        for _iter in 0..40 {
            path_hashes.clear();
            let st = TT.with(|tt_cell| {
                let mut tt = tt_cell.borrow_mut();
                let (st, nb) = self.dfs(
                    cube,
                    0,
                    &mut path,
                    bound,
                    t0,
                    &mut tt,
                    None,
                    &mut path_hashes,
                    solve_id,
                );
                (st, nb)
            });
            match st {
                (SearchStatus::Found, _) => return Some(path),
                (SearchStatus::NotFound, nb) => {
                    if nb >= 100 || t0.elapsed().as_secs() >= SOLVE_TIMEOUT_SECS {
                        return None;
                    }
                    bound = nb;
                }
            }
        }
        None
    }

    fn dfs(
        &self,
        cube: &Cube,
        g: u8,
        path: &mut Vec<Move>,
        bound: u8,
        t0: std::time::Instant,
        tt: &mut [TtEntry],
        last_face: Option<u8>,
        path_hashes: &mut Vec<u64>,
        solve_id: u16,
    ) -> (SearchStatus, u8) {
        thread_local! {
            static NODE_COUNTER: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
        }
        let count = NODE_COUNTER.with(|c| {
            let val = c.get() + 1;
            c.set(val);
            val
        });
        if count & 0xFF == 0 && t0.elapsed().as_secs() >= SOLVE_TIMEOUT_SECS {
            return (SearchStatus::NotFound, u8::MAX);
        }

        let hash = cube.zobrist_hash();
        let remaining_depth = bound.saturating_sub(g);

        if remaining_depth <= 6 {
            if let Some(d) = self.solved_table.get_distance(hash) {
                if g + d <= bound {
                    let mut cur = *cube;
                    let mut rem = d;
                    while rem > 0 {
                        let mut ok = false;
                        for &m in &ALL_MOVES {
                            let mut nc = cur;
                            nc.apply_move(m);
                            if let Some(nd) = self.solved_table.get_distance(nc.zobrist_hash()) {
                                if nd == rem - 1 {
                                    path.push(m);
                                    cur = nc;
                                    rem -= 1;
                                    ok = true;
                                    break;
                                }
                            }
                        }
                        if !ok {
                            break;
                        }
                    }
                    return (SearchStatus::Found, bound);
                } else {
                    return (SearchStatus::NotFound, g + d);
                }
            }
        }

        let ti = (hash as usize) & (tt.len() - 1);
        if tt[ti].hash == hash && tt[ti].solve_id == solve_id && tt[ti].depth >= remaining_depth {
            return (SearchStatus::NotFound, g.saturating_add(tt[ti].value));
        }

        let cpdb = self.corner_pdb.lookup(cube);
        let epdb = self.edge6_pdb.lookup(cube);
        let mut h = cpdb.max(epdb);
        if tt[ti].hash == hash && tt[ti].solve_id == solve_id && tt[ti].value > h {
            h = tt[ti].value;
        }

        let mut f = g.saturating_add(h);
        if f > bound {
            return (SearchStatus::NotFound, f);
        }

        if let Some(ref nnue) = self.heuristic {
            let raw = nnue.predict(&cube.to_features());
            let adj = (raw * 0.7).floor().max(0.0) as u8;
            if adj > h {
                h = adj;
                f = g.saturating_add(h);
                if f > bound {
                    return (SearchStatus::NotFound, f);
                }
            }
        }

        if path_hashes.contains(&hash) {
            return (SearchStatus::NotFound, u8::MAX);
        }
        path_hashes.push(hash);

        let mut children = [(0u8, Move::U, 0u8, Cube::new()); 18];
        let mut nchildren = 0;
        for &m in &ALL_MOVES {
            let cf = m.face();
            if let Some(pf) = last_face {
                if pf == cf {
                    continue;
                }
                if pf % 2 == 1 && cf == pf - 1 {
                    continue;
                }
            }
            let mut nc = *cube;
            nc.apply_move(m);
            let ch = self.corner_pdb.lookup(&nc).max(self.edge6_pdb.lookup(&nc));
            children[nchildren] = (ch, m, cf, nc);
            nchildren += 1;
        }

        children[..nchildren].sort_unstable_by_key(|&(ch, _, _, _)| ch);

        let mut min_exc = u8::MAX;
        for i in 0..nchildren {
            let (_, m, _, nc) = children[i];
            path.push(m);
            let (st, nb) = self.dfs(
                &nc,
                g + 1,
                path,
                bound,
                t0,
                tt,
                Some(m.face()),
                path_hashes,
                solve_id,
            );
            match st {
                SearchStatus::Found => {
                    path_hashes.pop();
                    return (SearchStatus::Found, bound);
                }
                SearchStatus::NotFound => {
                    if nb < min_exc {
                        min_exc = nb;
                    }
                    path.pop();
                }
            }
        }

        if min_exc != u8::MAX {
            let h_exc = min_exc.saturating_sub(g);
            if tt[ti].hash != hash || tt[ti].solve_id != solve_id || remaining_depth >= tt[ti].depth
            {
                tt[ti] = TtEntry {
                    hash,
                    depth: remaining_depth,
                    value: h_exc,
                    solve_id,
                };
            }
        }

        path_hashes.pop();
        (SearchStatus::NotFound, min_exc)
    }
}
