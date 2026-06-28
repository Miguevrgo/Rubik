use crate::{cube::Cube, search::ida, tables::SearchData};

mod cube;
mod search;
mod tables;

fn main() {
    let mut cube = Cube::new();
    let moves = cube.shuffle(2);
    println!("{moves:?}");
    let mut data = SearchData::new();
    data.time_ts = 3000;
    while !cube.is_solved() {
        println!("{cube}");
        ida(&cube, &mut data);
        println!("{:?}", data.best_move);
        cube.apply_move(data.best_move);
    }
}
