use std::{fmt, time::Instant};

use crate::{
    cube::Move,
    search::{INF, MAX_DEPTH, SOLVED},
};

pub struct SearchData {
    // Search Control
    pub timing: Instant,
    pub time_ts: u128,
    pub stop: bool,
    pub depth: u8,

    // Data
    pub ply: usize,
    pub nodes: u64,
    pub best_move: Move,
    pub eval: i32,

    pub stack: Vec<u64>,
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
            best_move: Move::NULL,
            eval: -INF,
            stack: Vec::with_capacity(16),
        }
    }

    pub fn start_search(&mut self) {
        self.depth = 1;
        self.stop = false;
        self.best_move = Move::NULL;
        self.ply = 0;
        self.nodes = 0;
        self.timing = Instant::now();
    }

    pub fn push(&mut self, hash: u64) {
        self.ply += 1;
        self.stack.push(hash);
    }

    pub fn pop(&mut self) {
        self.ply -= 1;
        self.stack.pop();
    }

    pub fn continue_search(&self) -> bool {
        let time = self.timing.elapsed().as_millis();
        time < self.time_ts
    }
}

impl fmt::Display for SearchData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let time = self.timing.elapsed().as_millis();
        let nps = (1000 * self.nodes as u128).checked_div(time).unwrap_or(0) as u64;

        if self.eval.abs() >= SOLVED - i32::from(MAX_DEPTH) {
            write!(
                f,
                "[+] Depth {} Solved {} Time {time} Nodes {} NPS {nps}",
                self.depth, self.ply, self.nodes,
            )
        } else {
            write!(
                f,
                "[+] Depth {} Score {} Time {time} Nodes {} NPS {nps}",
                self.depth, self.eval, self.nodes,
            )
        }
    }
}
