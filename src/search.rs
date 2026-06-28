use crate::{
    cube::{Cube, Move},
    tables::SearchData,
};

pub const INF: i32 = 2 << 16;
pub const SOLVED: i32 = INF << 2;
pub const MAX_DEPTH: u8 = 32;

pub fn ida(cube: &Cube, data: &mut SearchData) {
    data.start_search();

    while !data.stop {
        data.eval = negamax(cube, data.depth, -INF, INF, data);

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

fn negamax(cube: &Cube, depth: u8, mut alpha: i32, beta: i32, data: &mut SearchData) -> i32 {
    if data.stop || (data.nodes & 4095 == 0 && !data.continue_search()) {
        data.stop = true;
        return 0;
    }

    let mut best_move = Move::NULL;
    let mut best_score = -INF;

    if depth == 0 {
        return cube.evaluate(); // TODO:
    }

    data.push(cube.hash());

    for mv in Move::ALL {
        let mut new_cube = *cube;
        new_cube.apply_move(mv);

        data.nodes += 1;

        let score = negamax(&new_cube, depth - 1, -beta, -alpha, data);

        if score > best_score {
            alpha = alpha.max(score);
            best_score = score;
            best_move = mv;
        }

        if alpha >= beta {
            break;
        }
    }

    data.pop();

    if data.ply == 0 {
        data.best_move = best_move;
    }

    best_score
}
