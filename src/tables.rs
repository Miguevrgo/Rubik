use std::{fmt, time::Instant};

use crate::cube::Move;

pub struct SearchData {
    // Search Control
    pub timing: Instant,
    pub time_ts: u128,
    pub stop: bool,
    pub depth: u8,

    // Data
    pub ply: usize,
    pub nodes: u64,
    pub solution: Vec<Move>,
    pub solved: bool,

    pub tt: TranspositionTable,
}

impl SearchData {
    pub fn new() -> Self {
        Self {
            timing: Instant::now(),
            time_ts: 0,
            stop: false,
            depth: 0,

            ply: 0,
            nodes: 0,
            solved: false,
            solution: Vec::new(),
            tt: TranspositionTable::with_size_mb(32),
        }
    }

    pub fn start_search(&mut self) {
        self.depth = 1;
        self.stop = false;
        self.ply = 0;
        self.solved = false;
        self.solution.clear();
        self.nodes = 0;
        self.timing = Instant::now();
        self.tt.clear();
    }

    pub fn push(&mut self) {
        self.ply += 1;
    }

    pub fn pop(&mut self) {
        self.ply -= 1;
    }

    pub fn continue_search(&self) -> bool {
        let time = self.timing.elapsed().as_millis();
        time < self.time_ts
    }
}

#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct TTEntry {
    pub key: u64,
    pub depth: u8,
}

pub struct TranspositionTable {
    pub tt: Vec<TTEntry>,
}

impl TranspositionTable {
    pub fn with_size_mb(mb: usize) -> Self {
        let bytes = mb * 1_048_576;
        let entry_sz = std::mem::size_of::<TTEntry>();
        let len = (bytes / entry_sz).next_power_of_two();
        Self {
            tt: vec![TTEntry::default(); len],
        }
    }

    fn idx(&self, hash: u64) -> usize {
        // (Read Lemire Blog for explanation | Carp)
        ((hash as u128 * self.tt.len() as u128) >> 64) as usize
    }

    pub fn probe(&self, hash: u64) -> Option<u8> {
        let e = &self.tt[self.idx(hash)];
        (e.key == hash).then_some(e.depth)
    }

    pub fn insert(&mut self, hash: u64, depth: u8) {
        let idx = self.idx(hash);
        let slot = &mut self.tt[idx];

        if slot.key != hash || depth >= slot.depth {
            slot.key = hash;
            slot.depth = depth;
        }
    }

    pub fn clear(&mut self) {
        for entry in self.tt.iter_mut() {
            entry.key = 0;
            entry.depth = 0;
        }
    }
}

impl fmt::Display for SearchData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let time = self.timing.elapsed().as_millis();
        let nps = (1000 * self.nodes as u128).checked_div(time).unwrap_or(0) as u64;

        if self.solved {
            write!(
                f,
                "[+] Depth {} Solved {} Time {time} Nodes {} NPS {nps}",
                self.depth, self.ply, self.nodes,
            )
        } else {
            write!(
                f,
                "[+] Depth {} Time {time} Nodes {} NPS {nps}",
                self.depth, self.nodes,
            )
        }
    }
}
