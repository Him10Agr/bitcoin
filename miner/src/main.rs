use std::{env, usize};
use std::process::exit;

use btclib::types::Block;
use btclib::util::Saveable;

fn main() {
    //"block path" "steps count" -> cli arg
    let (path, steps) = if let(Some(arg1), Some(arg2)) = (env::args().nth(1), env::args().nth(2)) {
        (arg1, arg2)
    } else {
        eprintln!("Usage: miner <block_file> <steps>");
        exit(1);
    };

    //parse steps count
    let steps: usize = if let Ok(s @ 1..=usize::MAX) = steps.parse() {
        s
    } else {
        eprintln!("<steps> should be a positive integer");
        exit(1);
    };


    //load the block
    let og_block = Block::load_from_file(path).expect("Failed to load block");
    let mut block = og_block.clone();

    while !block.header.mine(steps) {
        println!("mining....");
    }

    //original block and minted block printout
    println!("Original: {:#?}", og_block);
    println!("Hash: {}", og_block.header.hash());
    println!("Final: {:#?}", block);
    println!("Hash: {}", block.header.hash());

}