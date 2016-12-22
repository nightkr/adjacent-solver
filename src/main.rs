extern crate rand;

mod unequal;

use std::io::Write;
use std::fs::File;

fn main() {
    let ls = unequal::LatinSquare::random(7);
    println!("{}", ls.pprint());

    let mut uls = unequal::UnequalLatinSquare::from_latin_square(ls);
    println!("Tatham Puzzle ID: {}", uls.tatham_puzzle_id());
    println!("{}", uls.pprint());
    let mut step = 1;
    while !uls.solved() {
        uls.solve_step();
        println!("{}", uls.pprint());
        let mut file = File::create(format!("steps/step.{}", step)).unwrap();
        write!(file, "{}", uls.to_tatham_save()).unwrap();
        step += 1;
    }
}
