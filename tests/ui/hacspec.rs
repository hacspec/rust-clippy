#![warn(clippy::hacspec)]

// Get hacspec and all depending crates.
extern crate hacspec;
hacspec::hacspec_crates!();

use hacspec::*;
hacspec_imports!();

// These are type aliases for convenience
type State = [u32; 16];

// These are actual types; fixed-length arrays.
bytes!(StateBytes, 64);
bytes!(IV, 12);
bytes!(Key, 32);

pub fn state_to_bytes(x: State) -> StateBytes {
    let mut r = StateBytes::new();
    for i in 0..x.len() {
        let bytes = Bytes::from_u32l(x[i]);
        r[i * 4] = bytes[3];
        r[i * 4 + 1] = bytes[2];
        r[i * 4 + 2] = bytes[1];
        r[i * 4 + 3] = bytes[0];
    }
    r
}

pub fn main() {}
