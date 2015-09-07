#![feature(test)]

#[macro_use]

extern crate itertools;

extern crate argparse;
extern crate ansi_term;
extern crate rustc_serialize;
extern crate time;

mod space;
mod identity;
mod motion;
mod life;
mod mind;

use std::io;
use std::io::Write;
use std::process;

use argparse::{ArgumentParser, Store};
use rustc_serialize::json;
use time::*;

use identity::Agent;
use life::{WorldState, Commit, Patch};
use mind::kickoff;



fn forecast(world: WorldState, depth: u8) -> (Vec<(Commit, f32)>, Duration) {
    let start_thinking = time::get_time();
    let forecasts: Vec<(Commit, f32)> = kickoff(world, depth);
    let stop_thinking = time::get_time();
    let thinking_time = stop_thinking - start_thinking;
    (forecasts, thinking_time)
}


fn oppose(in_medias_res: WorldState, depth: u8) -> (Commit, Duration) {
    let (mut forecasts, thinking_time) = forecast(in_medias_res, depth);
    let (determination, _karma) = forecasts
        .pop()
        .expect("Cannot oppose with no moves!");
    (determination, thinking_time)
}


#[derive(RustcEncodable, RustcDecodable)]
struct Postcard {
    world: String,
    patch: Patch,
    hospitalization: Option<Agent>,
    thinking_time: u64
}


fn correspond(reminder: String, depth: u8) -> String {
    let world = WorldState::reconstruct(reminder);
    let (commit, sidereal) = oppose(world, depth);
    let postcard = Postcard {
        world: commit.tree.preserve(),
        patch: commit.patch,
        hospitalization: commit.hospitalization,
        thinking_time: sidereal.num_milliseconds() as u64
    };
    json::encode(&postcard).unwrap()
}


fn the_end() {
    println!("THE END");
    process::exit(0);
}


fn main() {
    // Does argparse not offer a way to Store an argument (not a
    // hardcoded value) into an Option? Contribution opportunity if so??
    //
    // For now, use 0 like None.
    let mut lookahead_depth: u8 = 0;
    let mut postcard: String = "".to_string();
    {
        let mut parser = ArgumentParser::new();
        parser.set_description("Leafline: an oppositional strategy game engine");
        parser.refer(&mut lookahead_depth).add_option(
            &["--lookahead"], Store,
            "rank moves using AI minimax lookahead this deep."
        );
        parser.refer(&mut postcard).add_option(
            &["--correspond"], Store,
            "just output the serialization of the AI's top move in response \
             to the given serialized world-state"
        );
        parser.parse_args_or_exit();
    }

    if !postcard.is_empty() {
        println!("{}", correspond(postcard, lookahead_depth));
        process::exit(0);
    }

    let mut world = WorldState::new();
    let mut premonitions: Vec<Commit>;
    loop {
        match lookahead_depth {
            // XXX can we unify the scored and unscored logic? Useful
            // not only on general DRYness principles, but perhaps also
            // toward supporting human vs. computer playish things rather
            // than just advising every movement
            0 => {
                premonitions = world.lookahead();
                if premonitions.len() == 0 {
                    // XXX distinguish between stalemate and
                    // checkm^H^H^H^H^H^Hultimate endangerment
                    the_end();
                }
                world.display();
                println!("");
                for (index, premonition) in premonitions.iter().enumerate() {
                    println!("{:>2}. {}", index, premonition)
                }
            },
            _ => {
                let (forecasts,
                     thinking_time) = forecast(world, lookahead_depth);
                world.display();
                println!(
                    "(scoring alternatives {} levels deep took {} ms)",
                    lookahead_depth, thinking_time.num_milliseconds()
                 );
                for (index, prem_score) in forecasts.iter().enumerate() {
                    println!("{:>2}. {} (score {})", index, prem_score.0, prem_score.1);
                }
                premonitions = vec!();
                for prem_score in forecasts {
                    premonitions.push(prem_score.0);
                }

                if premonitions.len() == 0 {
                    the_end();
                }
            }

        }
        loop {
            print!("\nSelect a move>> ");
            io::stdout().flush().ok().expect("couldn't flush stdout");
            let mut input_buffer = String::new();
            io::stdin()
                .read_line(&mut input_buffer)
                .ok().expect("couldn't read input");

            if input_buffer.trim() == "quit" {
                the_end();
            }

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
