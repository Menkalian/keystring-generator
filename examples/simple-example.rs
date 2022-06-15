extern crate keystring_generator;

use std::path::PathBuf;
use keystring_generator::generate;

fn main() {
    let input_path = PathBuf::new().join("examples/simple.keys");
    generate(&input_path).unwrap();
}