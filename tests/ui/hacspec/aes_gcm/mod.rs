#![allow(clippy::all)]
#![deny(clippy::hacspec)]
#![deny(clippy::hacspec_macros)]

extern crate contracts;
extern crate hacspec;
extern crate rand;

pub mod aes;
mod aesgcm;
pub mod gf128;

pub use aesgcm::*;

fn main() {}
