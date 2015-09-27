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
mod landmark;
mod life;
mod mind;


use std::io;
use std::io::Write;
use std::process;

use argparse::{ArgumentParser, Store, StoreTrue};
use rustc_serialize::json;
use time::*;

use identity::{Agent, Team};
use life::{WorldState, Commit, Patch};
use mind::{kickoff, iterative_deepening_kickoff};


enum LookaheadBound {
    Depth(u8),
    Seconds(u8)
}

impl LookaheadBound {
    pub fn duration(&self) -> Duration {
        match *self {
            LookaheadBound::Seconds(secs) => Duration::seconds(secs as i64),
            _ => moral_panic!("`duration()` called on non-Seconds \
                               LookaheadBound variant")
        }
    }
}

fn forecast(world: WorldState, bound: LookaheadBound)
            -> (Vec<(Commit, f32)>, u8, Duration) {
    let start_thinking = time::get_time();
    let forecasts;
    let depth;
    match bound {
        LookaheadBound::Depth(ds) => {
            forecasts = kickoff(&world, ds, false);
            depth = ds;
        },
        LookaheadBound::Seconds(_) => {
            let (fs, ds) = iterative_deepening_kickoff(
                &world, bound.duration(), false);
            forecasts = fs;
            depth = ds;
        }
    }
    let stop_thinking = time::get_time();
    let thinking_time = stop_thinking - start_thinking;
    (forecasts, depth, thinking_time)
}


#[derive(RustcEncodable, RustcDecodable)]
struct Postcard {
    world: String,
    patch: Patch,
    hospitalization: Option<Agent>,
    thinking_time: u64,
    depth: u8,
    counterreplies: Vec<Patch>,
}

#[derive(RustcEncodable, RustcDecodable)]
struct LastMissive {
    the_triumphant: Option<Team>
}

fn correspondence(reminder: String, bound: LookaheadBound) -> String {
    let in_medias_res = WorldState::reconstruct(reminder);
    let (mut forecasts, depth, sidereal) = forecast(in_medias_res, bound);

    if !forecasts.is_empty() {
        let (determination, _karma) = forecasts.swap_remove(0);
        // XXX TODO FIXME: this doesn't distinguish amongst ascensions
        // (and we can imagine somewhat contrived situations where only
        // some of them are admissible movements)
        let counterreplies = determination.tree.lookahead()
            .iter().map(|c| c.patch).collect::<Vec<_>>();
        if counterreplies.is_empty() {
            if determination.tree.in_critical_endangerment(Team::Orange) {
                return json::encode(
                    &LastMissive { the_triumphant: Some(Team::Blue) }).unwrap()
            } else {
                return json::encode(
                    &LastMissive { the_triumphant: None }).unwrap()
            }
        }
        let postcard = Postcard {
            world: determination.tree.preserve(),
            patch: determination.patch,
            hospitalization: determination.hospitalization,
            thinking_time: sidereal.num_milliseconds() as u64,
            depth: depth,
            counterreplies: counterreplies,
        };
        json::encode(&postcard).unwrap()
    } else {
        if in_medias_res.in_critical_endangerment(Team::Blue) {
            json::encode(&LastMissive { the_triumphant: Some(Team::Orange) }).unwrap()
        } else {
            json::encode(&LastMissive { the_triumphant: None }).unwrap()
        }
    }
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
    let mut from: String = "".to_owned();
    let mut correspond: bool = false;
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
        parser.refer(&mut correspond).add_option(
            &["--correspond"],
            StoreTrue,
            "just output the serialization of the AI's top \
             move");
        parser.refer(&mut from).add_option(
            &["--from"],
            Store,
            "start a game from the given book of preservation runes");
        parser.parse_args_or_exit();
    }

    if correspond {
        let bound;
        if lookahead_depth != 0 && lookahead_seconds == 0 {
            bound = LookaheadBound::Depth(lookahead_depth)
        } else if lookahead_seconds != 0 && lookahead_depth == 0 {
            bound = LookaheadBound::Seconds(lookahead_seconds)
        } else {
            println!("`--correspond` requires exactly one of \
                      `--depth` and `--seconds`");
            process::exit(1);
        }
        println!("{}", correspondence(from, bound));
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
            // XXX TODO FIXME clean up duplication
            if lookahead_depth != 0 && lookahead_seconds == 0 {
                let (our_forecasts, _depth, thinking_time) = forecast(
                    world, LookaheadBound::Depth(lookahead_depth));
                forecasts = our_forecasts;
                println!("{}", world);
                println!(
                    "(scoring alternatives {} levels deep took {} ms)",
                    lookahead_depth, thinking_time.num_milliseconds()
                );
            } else if lookahead_seconds != 0 && lookahead_depth == 0 {
                let (our_forecasts, depth, thinking_time) = forecast(
                    world, LookaheadBound::Seconds(lookahead_seconds));
                forecasts = our_forecasts;
                println!("{}", world);
                println!(
                    "(scoring alternatives {} levels deep took {} ms)",
                    depth, thinking_time.num_milliseconds()
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

#[test]
mod tests {
    use super::{correspondence, LastMissive, LookaheadBound};

    #[test]
    fn concerning_correspondence_victory_conditions() {
        let blue_concession = correspondence(
            "R6k/6pp/8/8/8/8/8/8 b -".to_owned(),
            LookaheadBound::Depth(2)
        );
        assert_eq!("{\"the_triumphant\":\"Orange\"}".to_owned(),
                   blue_concession);
    }

}
