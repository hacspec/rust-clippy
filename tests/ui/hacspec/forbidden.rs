#![allow(clippy::all)]
#![deny(clippy::hacspec)]

extern crate hacspec;
extern crate rand;
// forbidden import, not inspected for now since it should be in the Cargo.toml
extern crate serde;

use hacspec::prelude::*;

// forbidden import
//TODO add a way to allow this in tests
use serde::Serialize;

bytes!(Block, 6);

///
/// Collection of forbidden uses of rust in hacspec subset

pub enum ForbiddenEnum {
    ForbiddenReference(&'static i32),
}

pub struct ForbiddenStruct {
    forbidden_reference: &'static i32,
    y: i32,
}

// mut static vec = vec![]
type ForbiddenPointer<'a> = &'a ForbiddenStruct;

// should pass
fn input(inp: ByteSeq) -> () {
    let b = inp;
}

fn main() {
    // #[clippy::author]

    // should pass
    let b: Block = Block::new();
    let v: Vec<u32> = Vec::<u32>::new();
}
