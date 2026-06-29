use crate::{
    cube::{Cube, Move},
    tables::SearchData,
};

pub fn ida(cube: &Cube, data: &mut SearchData) {
    data.start_search();

    while !data.stop {
        if dfs(cube, data.depth, data, Move::NULL) {
            data.stop = true;
            data.solved = true;
            data.solution.reverse();
        };

        println!("{data}");
        data.depth += 1;

        if data.stop {
            break;
        } else if data.timing.elapsed().as_millis() > data.time_ts {
            data.stop = true;
        }
    }
}

fn dfs(cube: &Cube, depth: u8, data: &mut SearchData, last_move: Move) -> bool {
    if data.stop || (data.nodes & 4095 == 0 && !data.continue_search()) {
        data.stop = true;
        return false;
    }

    let key = cube.hash();
    if let Some(saved_depth) = data.tt.probe(key)
        && saved_depth >= depth
    {
        return false;
    }

    if depth == 0 {
        return cube.is_solved(); // TODO: evaluate
    }

    data.push();

    for mv in Move::gen_moves(last_move) {
        let mut new_cube = *cube;
        new_cube.apply_move(mv);

        data.nodes += 1;

        if dfs(&new_cube, depth - 1, data, mv) {
            data.solution.push(mv);
            data.pop();
            return true;
        }
    }

    data.pop();
    data.tt.insert(key, depth);

    false
}
