use crate::cube::Cube;

mod cube;
mod search;

fn main() {
    let cube = Cube::new();
    println!("{cube:?}");
}
