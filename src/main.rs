#[macro_use]
extern crate itertools;

extern crate ansi_term;


mod space;
mod identity;
mod motion;
mod life;

use space::{Locale, Pinfield};
use identity::{Team, JobDescription, Agent};
use motion::{PONY_MOVEMENT_TABLE, FIGUREHEAD_MOVEMENT_TABLE};
use life::{WorldState, Patch, Commit};


fn main() {
    let world = WorldState::new();
    world.display();
    println!("");
    for (index, premonition) in world.lookahead().iter().enumerate() {
        println!("{}. {}", index, premonition)
    }
}
