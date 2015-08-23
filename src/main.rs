#[macro_use]
extern crate itertools;

extern crate argparse;
extern crate ansi_term;
extern crate time;

mod space;
mod identity;
mod motion;
mod life;
mod mind;

use std::io;
use std::io::Write;

use argparse::{ArgumentParser, Store};
use time::*;

use space::{Locale, Pinfield};
use identity::{Team, JobDescription, Agent};
use motion::{PONY_MOVEMENT_TABLE, FIGUREHEAD_MOVEMENT_TABLE};
use life::{WorldState, Patch, Commit};
use mind::negamax_kickoff;


fn main() {
    // Does argparse not offer a way to Store an argument (not a
    // hardcoded value) into an Option? Contribution opportunity if so??
    //
    // For now, use 0 like None.
    let mut lookahead_depth: u8 = 0;

    {
        let mut parser = ArgumentParser::new();
        parser.set_description("Leafline: an oppositional strategy game engine");
        parser.refer(&mut lookahead_depth).add_option(
            &["--lookahead"], Store,
            "rank moves using AI minimax lookahead this deep."
        );
        parser.parse_args_or_exit();
    }

    let mut world = WorldState::new();
    let mut premonitions: Vec<Commit>;
    loop {
        match lookahead_depth {
            0 => {
                premonitions = world.lookahead();
                world.display();
                println!("");
                for (index, premonition) in premonitions.iter().enumerate() {
                    println!("{:>2}. {}", index, premonition)
                }
            },
            _ => {
                let start_thinking = time::get_time();
                let forecasts = negamax_kickoff(world, lookahead_depth);
                let stop_thinking = time::get_time();
                let thinking_time = stop_thinking - start_thinking;
                world.display();
                println!(
                    "(scoring alternatives {} levels deep took {} ms)",
                    lookahead_depth, thinking_time.num_milliseconds()
                 );
                for (index,
                     &(premonition, score)) in forecasts.iter().enumerate() {
                    println!("{:>2}. {} (score {})", index, premonition, score);
                }
                premonitions = forecasts.iter().map(|t| t.0).collect::<Vec<_>>();
            }

        }
        loop {
            print!("\nSelect a move>> ");
            io::stdout().flush().ok().expect("couldn't flush stdout");
            let mut input_buffer = String::new();
            io::stdin()
                .read_line(&mut input_buffer)
                .ok().expect("couldn't read input");
            let choice: usize = match input_buffer.trim().parse() {
                Ok(i) => i,
                Err(e) => {
                    println!("Error parsing choice: {:?}. Try again.", e);
                    continue;
                }
            };
            if choice < premonitions.len() {
                world = premonitions[choice].tree;
                break;
            } else {
                println!("{} isn't among the choices. Try again.",
                         choice);
            }
        }
    }
}
