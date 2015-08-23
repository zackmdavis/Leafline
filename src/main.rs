#[macro_use]
extern crate itertools;

extern crate ansi_term;


mod space;
mod identity;
mod motion;
mod life;

use std::io;
use std::io::Write;

use space::{Locale, Pinfield};
use identity::{Team, JobDescription, Agent};
use motion::{PONY_MOVEMENT_TABLE, FIGUREHEAD_MOVEMENT_TABLE};
use life::{WorldState, Patch, Commit};


fn main() {
    let mut world = WorldState::new();
    loop {
        let premonitions = world.lookahead();
        world.display();
        println!("");
        for (index, premonition) in premonitions.iter().enumerate() {
            println!("{}. {}", index, premonition)
        }
        print!("\nSelect a move>> ");
        io::stdout().flush().ok().expect("couldn't flush stdout");
        let mut input_buffer = String::new();
        io::stdin()
            .read_line(&mut input_buffer)
            .ok().expect("couldn't read input");
        let choice: usize = input_buffer.trim().parse().ok().expect(
            "couldn't parse move choice");
        world = premonitions[choice].tree;
        world.to_move = world.to_move.opposition();
    }
}
