use crate::{
    cube::{Cube, Move},
    tables::SearchData,
};

pub const INF: i32 = 2 << 16;
pub const SOLVED: i32 = INF << 2;
pub const MAX_DEPTH: u8 = 32;

fn find_best_move(cube: &mut Cube, data: &mut SearchData) {
    data.start_search();

    while !data.stop {
        data.eval = ida(cube, -INF, INF, data);

        if data.stop {
            break;
        } else if data.timing.elapsed().as_millis() > data.time_ts
            || data.eval >= SOLVED - i32::from(MAX_DEPTH)
        {
            data.stop = true;
        }

        println!("{data}");
        data.depth += 1;
    }
}

fn ida(cube: &mut Cube, alpha: i32, beta: i32, data: &mut SearchData) -> i32 {
    0
}
