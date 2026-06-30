use crate::{cube::Cube, search::ida, tables::SearchData};

mod cube;
mod search;
mod tables;

fn main() {
    let mut cube = Cube::new();
    let moves = cube.shuffle(9);
    println!("{moves:?}");
    let mut data = SearchData::new();
    data.time_ts = 8000;

    ida(&cube, &mut data);
    println!("{:?}", data.solution);
}
