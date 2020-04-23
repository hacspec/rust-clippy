#![allow(clippy::all)]
#![deny(clippy::hacspec)]
#![deny(clippy::hacspec_macros)]

extern crate hacspec;
extern crate rand;

use hacspec::prelude::*;


// should pass
bytes!(Block, 6);

fn main() {

    // should pass
    let b:Block = Block::new();
    // #[clippy::author]
    // vec shouldn't pass
    let v:Vec<u32> = vec![0,1,2];

    // dbg! shouldn't pass
    dbg!("forbidden");
}