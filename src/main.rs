//! Leafline is an oppositional strategy game engine

extern crate ansi_term;

mod space;
mod identity;
mod motion;
mod life;

use space::{Locale, Pinfield};
use identity::{Team, JobDescription, Agent};
use motion::{PONY_MOVEMENT_TABLE, FIGUREHEAD_MOVEMENT_TABLE};
use life::{WorldState};


fn main() {
    let arena = WorldState::new();
    arena.display();
    println!("");
}
