#![feature(augmented_assignments, hashmap_hasher, non_ascii_idents, plugin)]

#![plugin(clippy)]

#![cfg(feature = "embassy")]

extern crate argparse;
extern crate ansi_term;
#[macro_use] extern crate itertools;
extern crate libc;
#[macro_use] extern crate log;
extern crate lru_cache;
extern crate rustc_serialize;
extern crate time;
extern crate twox_hash;


#[macro_use] mod sorceries;
mod space;
mod identity;
mod motion;
mod landmark;
mod life;
mod mind;
mod substrate;

use std::collections::HashMap;
use std::ffi::CStr;
use std::sync::{Arc, Mutex};

use life::{Patch, WorldState};
use mind::potentially_timebound_kickoff;


#[derive(Debug)]
#[repr(C)]
pub struct Scoring {
    pub movement: [u8; 10],
    pub score: f32,
}


#[no_mangle]
pub extern "C" fn score(preservation_runes: *const libc::c_char, depth: u8,
    output_scorings: *mut Scoring) {
    let experience_table: HashMap<Patch, u32> = HashMap::new();
    let intuition_bank = Arc::new(Mutex::new(experience_table));

    let buffer = unsafe { CStr::from_ptr(preservation_runes).to_bytes() };
    let scan = String::from_utf8(buffer.to_vec()).unwrap();

    let world = WorldState::reconstruct(scan);

    if let Some(result) = potentially_timebound_kickoff(
            &world, depth, None, true, None, intuition_bank, 1.0) {
        let mut output = unsafe {
            std::slice::from_raw_parts_mut(output_scorings, 60)
        };

        for (i, c_s_v) in result.into_iter().enumerate() {
            let (commit, score, _variation) = c_s_v;
            let mut movement = [0u8; 10];
            for (c, byte) in commit.pagan_movement_rune()
                                   .as_bytes()
                                   .iter()
                                   .enumerate() {
                movement[c] = *byte;
            }
            output[i] = Scoring {
                movement: movement,
                score: score,
            }
        }
    }

}
