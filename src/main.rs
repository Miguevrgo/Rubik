use crate::cube::Cube;

mod cube;
mod search;

fn main() {
    let mut cube = Cube::new();
    println!("Solved Cube:");
    println!("{cube:?}");

    println!("\nAfter R and U':");
    cube.right::<false>();
    cube.up::<true>();
    println!("{cube:?}");
}
