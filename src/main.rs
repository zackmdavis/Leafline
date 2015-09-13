#![feature(test)]
#![feature(non_ascii_idents)]
#![feature(plugin)]
#![plugin(clippy)]
#![allow(unused_features)]


#[macro_use]
extern crate itertools;

extern crate argparse;
extern crate ansi_term;
extern crate rustc_serialize;
extern crate time;


#[macro_use]
mod macros;

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
use mind::{kickoff, iterative_deepening_kickoff};


fn forecast(world: WorldState,
            depth: Option<u8>, seconds: Option<u8>)
            -> (Vec<(Commit, f32)>, Duration) {
    let start_thinking = time::get_time();
    let forecasts;
    if depth.is_some() && seconds.is_none() {
        forecasts = kickoff(&world, depth.unwrap(), false);
    } else if seconds.is_some() && depth.is_none() {
        forecasts = iterative_deepening_kickoff(
            &world, Duration::seconds(seconds.unwrap() as i64), false);
    } else {
        moral_panic!("both `depth` and `seconds` supplied, \
                      rather than only one of these");
    }
    let stop_thinking = time::get_time();
    let thinking_time = stop_thinking - start_thinking;
    (forecasts, thinking_time)
}


fn oppose(in_medias_res: WorldState, depth: u8) -> (Commit, Duration) {
    let (mut forecasts, thinking_time) = forecast(
        in_medias_res, Some(depth), None);
    let determination_and_karma;
    if !forecasts.is_empty() {
        determination_and_karma = forecasts.swap_remove(0);
    } else {
        // XXX TODO FIXME: during actual gameplay, we don't want to panic
        panic!("Cannot oppose with no moves");
    }
    let (determination, _karma) = determination_and_karma;
    (determination, thinking_time)
}


#[derive(RustcEncodable, RustcDecodable)]
struct Postcard {
    world: String,
    patch: Patch,
    hospitalization: Option<Agent>,
    thinking_time: u64,
}


fn correspond(reminder: String, depth: u8) -> String {
    let world = WorldState::reconstruct(reminder);
    let (commit, sidereal) = oppose(world, depth);
    let postcard = Postcard {
        world: commit.tree.preserve(),
        patch: commit.patch,
        hospitalization: commit.hospitalization,
        thinking_time: sidereal.num_milliseconds() as u64,
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
    let mut lookahead_seconds: u8 = 0;
    let mut postcard: String = "".to_owned();
    let mut from: String = "".to_owned();
    {
        let mut parser = ArgumentParser::new();
        parser.set_description("Leafline: an oppositional strategy game engine");
        parser.refer(&mut lookahead_depth).add_option(
            &["--depth"],
            Store,
            "rank moves using AI minimax lookahead this deep");
        parser.refer(&mut lookahead_seconds).add_option(
            &["--seconds"],
            Store,
            "rank moves using AI minimax for about this many seconds");
        parser.refer(&mut postcard).add_option(
            &["--correspond"],
            Store,
            "just output the serialization of the AI's top \
             move in response to the given serialized \
             world-state");
        parser.refer(&mut from).add_option(
            &["--from"],
            Store,
            "start a game from the given book of preservation runes");
        parser.parse_args_or_exit();
    }

    if !postcard.is_empty() {
        println!("{}", correspond(postcard, lookahead_depth));
        process::exit(0);
    }

    let mut world: WorldState;
    if !from.is_empty() {
        world = WorldState::reconstruct(from);
    } else {
        world = WorldState::new();
    }
    let mut premonitions: Vec<Commit>;
    loop {
        if lookahead_depth == 0 && lookahead_seconds == 0 {
            premonitions = world.lookahead();
            if premonitions.is_empty() {
                // XXX TODO distinguish between deadlock and
                // ultimate endangerment
                the_end();
            }
            println!("{}", world);
            for (index, premonition) in premonitions.iter().enumerate() {
                println!("{:>2}. {}", index, premonition)
            }
        }
        else {
            let forecasts;
            if lookahead_depth != 0 && lookahead_seconds == 0 {
                let (our_forecasts, thinking_time) = forecast(
                    world, Some(lookahead_depth), None);
                forecasts = our_forecasts;
                println!("{}", world);
                println!(
                    "(scoring alternatives {} levels deep took {} ms)",
                    lookahead_depth, thinking_time.num_milliseconds()
                );
            } else if lookahead_seconds != 0 && lookahead_depth == 0 {
                let (our_forecasts, thinking_time) = forecast(
                    world, None, Some(lookahead_seconds));
                forecasts = our_forecasts;
                println!("{}", world);
                println!(
                    // XXX we want to know how many depths we got out
                    // of those seconds
                    "(scoring alternatives took {} ms)",
                    thinking_time.num_milliseconds()
                );
            } else {
                println!("Supply only one of `--seconds` and `--depth`.");
                process::exit(1);
            }

            for (index, prem_score) in forecasts.iter().enumerate() {
                println!("{:>2}. {} (score {:.1})",
                         index, prem_score.0, prem_score.1);
            }
            premonitions = vec!();
            for prem_score in forecasts {
                premonitions.push(prem_score.0);
            }

            if premonitions.is_empty() {
                the_end();
            }
        }

        loop {
            print!("\nSelect a move>> ");
            io::stdout().flush().ok().expect("couldn't flush stdout");
            let mut input_buffer = String::new();
            io::stdin()
                .read_line(&mut input_buffer)
                .ok()
                .expect("couldn't read input");

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
