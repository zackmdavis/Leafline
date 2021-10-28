#![feature(plugin, test)]

#![allow(unknown_lints)]

#![allow(clippy::if_not_else, unused_features, clippy::clone_on_ref_ptr, clippy::clunreadable_literal, mixed_script_confusables, confusable_idents)]
#![warn(missing_debug_implementations, missing_copy_implementations,
trivial_casts, trivial_numeric_casts,
unused_import_braces, unused_qualifications)]



extern crate argparse;
extern crate ansi_term;
#[cfg_attr(test, macro_use)]
extern crate itertools;
#[macro_use]
extern crate log;
extern crate lru_cache;
extern crate parking_lot;
extern crate time;
extern crate twox_hash;
extern crate fnv;
extern crate serde_json;
extern crate serde;

#[macro_use]
extern crate lazy_static;


#[macro_use]
mod sorceries;
mod space;
mod identity;
mod motion;
mod landmark;
mod life;
mod mind;
mod substrate;
mod uci;
// Unlikely Command Integration
mod test_landmark;

use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::process;

use ansi_term::Colour as Color;
use argparse::{ArgumentParser, Print, Store, StoreOption, StoreTrue};
use log::{LogLevelFilter, LogMetadata, LogRecord, SetLoggerError};
use time::{Duration, get_time};

use identity::{Agent, Team};
use life::{Commit, Patch, TransitPatch, WorldState};
use mind::{Variation, fixed_depth_sequence_kickoff, iterative_deepening_kickoff,
           kickoff, pagan_variation_format, Memory};
use substrate::memory_free;
use serde_json::to_string;
use serde::Serialize;

fn encode<T>(value: &T) -> String
    where  T: ?Sized + serde::ser::Serialize {
    to_string(value).unwrap()
}

struct DebugLogger;

impl DebugLogger {
    pub fn init() -> Result<(), SetLoggerError> {
        log::set_logger(|max_log_level| {
            max_log_level.set(LogLevelFilter::Debug);
            Box::new(DebugLogger)
        })
    }
}

impl log::Log for DebugLogger {
    fn enabled(&self, _metadata: &LogMetadata) -> bool {
        true
    }

    fn log(&self, record: &LogRecord) {
        // XXX: can't the open file handle live inside the DebugLogger struct?!
        let mut log_file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open("leafline.log")
            .expect("couldn't open log file?!");
        let log_message = format!("[{}] {}\n",
                                  time::now()
                                      .strftime("%Y-%m-%d %H:%M:%S.%f")
                                      .unwrap(),
                                  record.args());
        log_file.write_all(&log_message.into_bytes())
            .expect("couldn't write to log file?!");
    }
}


#[derive(Debug, Clone)]
enum LookaheadBound {
    Depth(u8, Option<u8>),
    DepthSequence(Vec<u8>),
    Seconds(u8),
}


impl LookaheadBound {
    pub fn duration(&self) -> Duration {
        match *self {
            LookaheadBound::Seconds(secs) => Duration::seconds(i64::from(secs)),
            _ => {
                moral_panic!("`duration()` called on non-Seconds LookaheadBound \
                              variant")
            }
        }
    }

    pub fn new_from_sequence_depiction(depiction: &str) -> Self {
        let depth_runes = depiction.split(',');
        let depth_sequence = depth_runes.map(|dd| {
            dd.parse::<u8>()
                .expect("couldn't parse depth \
                                                       sequence")
        })
            .collect::<Vec<_>>();
        LookaheadBound::DepthSequence(depth_sequence)
    }

    pub fn from_args(lookahead_depth: Option<u8>,
                     lookahead_extension: Option<u8>,
                     lookahead_depth_sequence: Option<String>,
                     lookahead_seconds: Option<u8>)
                     -> Result<Option<Self>, String> {
        let mut bound = None;
        let confirm_bound_is_none =
            |b: &Option<LookaheadBound>| -> Result<bool, String> {
                if b.is_some() {
                    Err("more than one of `--depth`, `--depth-sequence`, or \
                         `--seconds` was passed"
                        .to_owned())
                } else {
                    Ok(true)
                }
            };
        if let Some(depth) = lookahead_depth {
            confirm_bound_is_none(&bound)?;
            bound = Some(LookaheadBound::Depth(depth, lookahead_extension));
        }
        if let Some(sequence_depiction) = lookahead_depth_sequence {
            confirm_bound_is_none(&bound)?;
            bound = Some(LookaheadBound::new_from_sequence_depiction(
                &sequence_depiction));
        }
        if let Some(seconds) = lookahead_seconds {
            confirm_bound_is_none(&bound)?;
            bound = Some(LookaheadBound::Seconds(seconds));
        }
        Ok(bound)
    }
}

fn forecast<T: 'static + Memory>(world: WorldState, bound: LookaheadBound, déjà_vu_bound: f32)
                                 -> (Vec<(Commit, f32, T)>, u8, Duration) {
    let start_thinking = get_time();
    let forecasts;
    let depth;
    match bound {
        LookaheadBound::Depth(ds, es) => {
            forecasts = kickoff::<T>(&world, ds, es, false, déjà_vu_bound);
            depth = ds;
        }
        LookaheadBound::DepthSequence(ds) => {
            depth = *ds.last().unwrap();
            forecasts = fixed_depth_sequence_kickoff::<T>(
                &world, ds, false, déjà_vu_bound);
            // XXX TODO: if we're just returning a number, it should be the
            // lowest depth, but we should really report all of them
        }
        LookaheadBound::Seconds(_) => {
            let (fs, ds) = iterative_deepening_kickoff::<T>(
                &world, bound.duration(), false, déjà_vu_bound);
            forecasts = fs;
            depth = ds;
        }
    }
    let stop_thinking = get_time();
    let thinking_time = stop_thinking - start_thinking;
    (forecasts, depth, thinking_time)
}


#[derive(Serialize)]
struct Postcard {
    world: String,
    patch: TransitPatch,
    hospitalization: Option<Agent>,
    thinking_time: u64,
    depth: u8,
    counterreplies: Vec<TransitPatch>,
    rosetta_stone: String,
}

#[derive(Serialize)]
struct LastMissive {
    the_triumphant: Option<Team>,
}

#[allow(clippy::clcollapsible_if)]
fn correspondence(reminder: &str, bound: LookaheadBound, déjà_vu_bound: f32)
                  -> String {
    let in_medias_res = WorldState::reconstruct(reminder);
    let (mut forecasts, depth, sidereal) = forecast::<Patch>(in_medias_res,
                                                             bound,
                                                             déjà_vu_bound);

    if !forecasts.is_empty() {
        let (determination, _karma, _variation) = forecasts.swap_remove(0);
        // XXX TODO FIXME: this doesn't distinguish amongst ascensions
        // (and we can imagine somewhat contrived situations where only
        // some of them are admissible movements)
        let counterreplies = determination.tree
            .lookahead()
            .iter()
            .map(|c| TransitPatch::from(c.patch))
            .collect::<Vec<_>>();
        if counterreplies.is_empty() {
            if determination.tree.in_critical_endangerment(Team::Orange) {
                return encode(&LastMissive {
                    the_triumphant: Some(Team::Blue),
                })
            } else {
                return encode(&LastMissive { the_triumphant: None });
            }
        }
        let postcard = Postcard {
            world: determination.tree.preserve(),
            patch: TransitPatch::from(determination.patch),
            hospitalization: determination.hospitalization,
            thinking_time: sidereal.num_milliseconds() as u64,
            depth,
            counterreplies,
            rosetta_stone: determination.patch.abbreviated_pagan_movement_rune(),
        };
        encode(&postcard)
    } else if in_medias_res.in_critical_endangerment(Team::Blue) {
        encode(&LastMissive { the_triumphant: Some(Team::Orange) })
    } else {
        encode(&LastMissive { the_triumphant: None })
    }
}


fn the_end() {
    println!("THE END");
    process::exit(0);
}


fn main() {
    // Does argparse not offer an analogue of Python's argparse's
    // `add_mutually_exclusive_group`
    // (https://docs.python.org/3/library/argparse.html#mutual-exclusion)?
    // Contribution opportunity if so??
    let mut lookahead_depth: Option<u8> = None;
    let mut lookahead_extension: Option<u8> = None;
    // TODO CONSIDER: would argparse's Collect action be cleaner?
    let mut lookahead_depth_sequence: Option<String> = None;
    let mut lookahead_seconds: Option<u8> = None;
    let mut from_runes: Option<String> = None;
    let mut correspond: bool = false;
    let mut uci_dæmon: bool = false;
    let mut déjà_vu_bound: f32 = 2.0;
    let mut debug_logging: bool = false;
    {
        let mut parser = ArgumentParser::new();
        parser.set_description("Leafline: an oppositional strategy game engine");
        parser.refer(&mut lookahead_depth).add_option(
            &["--depth"],
            StoreOption,
            "rank moves using AI minimax lookahead this deep");
        parser.refer(&mut lookahead_extension).add_option(
            &["--quiet"],
            StoreOption,
            "search with quietness extension this deep");
        parser.refer(&mut lookahead_depth_sequence).add_option(
            &["--depth-sequence"],
            StoreOption,
            "rank moves using AI minimax lookahead to these depths");
        parser.refer(&mut lookahead_seconds).add_option(
            &["--seconds"],
            StoreOption,
            "rank moves using AI minimax for about this many seconds");
        parser.refer(&mut correspond).add_option(
            &["--correspond"],
            StoreTrue,
            "just output the serialization of the AI's top response and \
             legal replies thereto");
        parser.refer(&mut uci_dæmon).add_option(
            &["--uci", "--deamon", "--dæmon"],
            StoreTrue,
            "run Unlikely Command Integration dæmon for external driver play");
        parser.refer(&mut from_runes).add_option(
            &["--from"],
            StoreOption,
            "start a game from the given book of preservation runes");
        parser.refer(&mut déjà_vu_bound).add_option(
            &["--déjà-vu-bound", "--deja-vu-bound"],
            Store,
            "try to not store more entries in the déjà vu table than fit in \
             this many GiB of memory",
        );
        parser.refer(&mut debug_logging).add_option(
            &["--debug"],
            StoreTrue,
            "run with debug logging to file",
        );
        parser.add_option(&["--version", "-v"],
                          Print(env!("CARGO_PKG_VERSION").to_owned()), "diplay the version");
        parser.parse_args_or_exit();
    }

    if debug_logging {
        DebugLogger::init().expect("couldn't initialize logging?!")
    }

    if correspond {
        let bound_maybe_result = LookaheadBound::from_args(lookahead_depth,
                                                           lookahead_extension,
                                                           lookahead_depth_sequence,
                                                           lookahead_seconds);
        let bound = match bound_maybe_result {
            Ok(bound_maybe) => {
                match bound_maybe {
                    Some(bound) => bound,
                    None => {
                        moral_panic!("`--correspond` passed without exactly one \
                                      of `--depth`, `--depth-sequence`, or \
                                      `--seconds`")
                    }
                }
            }
            Err(error) => {
                moral_panic!(error)
            }
        };
        let from = from_runes.expect("`--correspond` requires `--from`");
        println!("{}", correspondence(&from, bound, déjà_vu_bound));
        process::exit(0);
    }

    if uci_dæmon {
        uci::dæmon();
        // ↑ dæmon will loop
        process::exit(0);
    }

    println!("Welcome to Leafline v. {}!", env!("CARGO_PKG_VERSION"));
    match memory_free() {
        Some(bytes) => {
            println!("Leafline substrate accountant detected {:.3} \
                     GiB of free memory.",
                     bytes.in_gib());
        }
        None => {
            println!("Could not detect amount of free memory! \
                      It is possible \
                      that you are struggling with an inferior nonfree \
                      operating system forced on you by your masters in \
                      Cupertino or Redmond");
        }
    }

    let mut world = match from_runes {
        Some(runes) => WorldState::reconstruct(&runes),
        None => WorldState::new(),
    };
    let mut premonitions: Vec<Commit>;
    let bound_maybe = LookaheadBound::from_args(lookahead_depth,
                                                lookahead_extension,
                                                lookahead_depth_sequence,
                                                lookahead_seconds)
        .unwrap();
    loop {
        match bound_maybe {
            None => {
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
            Some(ref bound) => {
                let (our_forecasts, depth, thinking_time) =
                    forecast::<Variation>(world, bound.clone(), déjà_vu_bound);
                let forecasts = our_forecasts;
                println!("{}", world);
                let depth_report = match *bound {
                    LookaheadBound::Depth(standard, Some(quietening)) => {
                        format!(
                            "at least {} and up to {}",
                            standard,
                            standard + quietening)
                    }
                    _ => format!("{}", depth),
                };
                println!("(scoring alternatives {} levels deep took {} ms)",
                         depth_report,
                         thinking_time.num_milliseconds());
                premonitions = Vec::new();
                for (index, sight) in forecasts.into_iter().enumerate() {
                    let (commit, score, variation) = sight;
                    println!("{:>2}: {} — score {} ‣ representative variation: {}",
                             index,
                             commit,
                             Color::Purple.bold()
                                 .paint(&format!("{:.1}", score)),
                             pagan_variation_format(&variation));
                    premonitions.push(commit);
                }

                if premonitions.is_empty() {
                    the_end();
                }
            }
        }

        loop {
            print!("\nSelect a move>> ");
            io::stdout().flush().expect("couldn't flush stdout");
            let mut input_buffer = String::new();
            io::stdin()
                .read_line(&mut input_buffer)
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
                println!("{} isn't among the choices. Try again.", choice);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{LookaheadBound, correspondence};
    use ::{Postcard, encode};
    use life::TransitPatch;
    use space::{RelaxedLocale, Locale};
    use identity::{Agent, JobDescription, Team};
    use LastMissive;

    #[test]
    fn concerning_correspondence_victory_conditions() {
        let blue_concession = correspondence("R6k/6pp/8/8/8/8/8/8 b - -",
                                             LookaheadBound::Depth(2, None),
                                             1.0);
        assert_eq!("{\"the_triumphant\":\"Orange\"}".to_owned(),
                   blue_concession);
    }

    #[test]
    fn test_serialize_postcard() {
        let p = Postcard {
            world: "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2".to_string(),
            patch: TransitPatch {
                star: Agent { team: Team::Orange, job_description: JobDescription::Scholar },
                whence: RelaxedLocale::from(Locale::new(1, 2)),
                whither: RelaxedLocale::from(Locale::new(3, 4)),
            },
            hospitalization: None,
            thinking_time: 123,
            depth: 4,
            counterreplies: vec![],
            rosetta_stone: "Bd5".to_string()
        };

        assert_eq!(r#"{
"world":"rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2",
"patch":{
"star":{
"team":"Orange",
"job_description":"Scholar"
},
"whence":{"rank":1,"file":2},
"whither":{"rank":3,"file":4}
},
"hospitalization":null,
"thinking_time":123,
"depth":4,
"counterreplies":[],
"rosetta_stone":"Bd5"
}"#.replace("\n", ""),
            encode(&p)
        );

    }

    #[test]
    fn test_serialize_last_missive() {
        let l1 = LastMissive { the_triumphant: None };
        assert_eq!(r#"{"the_triumphant":null}"#, encode(&l1));
        let l1 = LastMissive { the_triumphant: Some(Team::Orange) };
        assert_eq!(r#"{"the_triumphant":"Orange"}"#, encode(&l1));
    }
}
