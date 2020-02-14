#![allow(clippy::all)]
#![deny(clippy::hacspec)]

// Get hacspec and all depending crates.
extern crate hacspec;
hacspec::hacspec_crates!();

// Import hacspec and all needed definitions.
use hacspec::*;
hacspec_imports!();

// Import chacha20 and poly1305
extern crate hacspecs;
use hacspecs::chacha20;
use hacspecs::poly1305;
use chacha20::*;
use poly1305::*;

// TODO: can we do without borrow?
fn pad_aad_msg(aad: Bytes, msg: &Bytes) -> Bytes {
    let laad = aad.len();
    let lmsg = msg.len();
    let pad_aad = if laad % 16 == 0 {
        laad
    } else {
        16 * ((laad >> 4) + 1)
    };
    let pad_msg = if lmsg % 16 == 0 {
        lmsg
    } else {
        16 * ((lmsg >> 4) + 1)
    };
    let mut padded_msg = Bytes::new_len(pad_aad + pad_msg + 16);
    padded_msg.update(0, &aad);
    padded_msg.update(pad_aad, msg);
    padded_msg.update_raw(pad_aad + pad_msg, &(laad as u64).to_le_bytes());
    padded_msg.update_raw(pad_aad + pad_msg + 8, &(lmsg as u64).to_le_bytes());
    padded_msg
}

pub fn encrypt(key: Key, iv: IV, aad: Bytes, msg: Bytes) -> Result<(Bytes, Tag), String> {
    let key_block = block(key, 0, iv);
    let mac_key = Key::from(&key_block[0..32]);
    let cipher_text = match chacha(key, iv, msg) {
        Ok(c) => c,
        Err(r) => {
            println!("Error encrypting chacha20: {}", r);
            return Err(r);
        }
    };
    let padded_msg = pad_aad_msg(aad, &cipher_text);
    let tag = poly(padded_msg, mac_key);
    Ok((cipher_text, tag))
}

pub fn decrypt(
    key: Key,
    iv: IV,
    aad: Bytes,
    cipher_text: Bytes,
    tag: Tag,
) -> Result<Bytes, String> {
    let key_block = block(key, 0, iv);
    let mac_key = Key::from(&key_block[0..32]);
    let padded_msg = pad_aad_msg(aad, &cipher_text);
    let my_tag = poly(padded_msg, mac_key);
    if my_tag == tag {
        match chacha(key, iv, cipher_text) {
            Ok(c) => Ok(c),
            Err(r) => {
                println!("Error decrypting chacha20: {}", r);
                Err(r)
            }
        }
    } else {
        Err("Mac verification failed".to_string())
    }
}
