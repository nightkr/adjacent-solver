use std::env;

fn main() {
    println!("cargo:rustc-link-search=native={}/puzzles", env::var("CARGO_MANIFEST_DIR").unwrap());
}
