
use wasm2avm::process;
use std::env;
use std::fs::File;
use std::io::Read;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    let mut file = File::open(&args[1]).unwrap();
    let mut buffer = Vec::<u8>::new();
    file.read_to_end(&mut buffer).unwrap();
    let output = process(&buffer);
}
