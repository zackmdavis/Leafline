use std::collections::HashMap;
use std::io;

use time::Duration;

use mind::{kickoff, iterative_deepening_kickoff};
use life::WorldState;
use space::Locale;
use identity::Team;

// Unlikely Command Integration dæmon

pub fn dæmon() {
    let mut input_buffer = String::new();
    let mut world = WorldState::new();

    loop {
        input_buffer.clear();
        io::stdin().read_line(&mut input_buffer)
            .expect("expected to be able to read stdin");

        match input_buffer.as_ref() {
            "uci\n" => {
                println!("id name Leafline");
                println!("id author Zack M. Davis and friends");
                println!("uciok");
                continue;
            }
            "isready\n" => {
                println!("readyok");
                continue;
            }
            "ucinewgame\n" => {
                world = WorldState::new();
                continue;
            }
            "quit\n" => {
                break;
            }
            _ => { /* hopefully, 'position', or 'go' */ }
        }

        let mut tokens = input_buffer.split_whitespace();
        let command = tokens.next().expect("expected a command");
        match command {
            "position" => {
                // XXX HACK: let's assume our internal WorldState is correct
                // and just look at the last-move diff
                let diff = tokens.last()
                    .expect("expected a discernable last move");
                // XXX: we're going to need better movement-parsing than this
                // to play a full game (consider secret service)
                let whence = Locale::from_algebraic(&diff[0..2]);
                let whither = Locale::from_algebraic(&diff[2..4]);
                let premonitions = world.lookahead();
                for premonition in premonitions.iter() {
                    if premonition.patch.whence == whence &&
                        premonition.patch.whither == whither {
                            world = premonition.tree;
                    }
                }
            },
            "go" => {
                let mut options = HashMap::new();
                while let Some(key) = tokens.next() {
                    let value = tokens.next()
                        .expect("expected option value").parse::<usize>()
                        .expect("expected an integer literal");
                    options.insert(key, value);
                }

                let mut forecasts;
                if let Some(depth) = options.get("depth") {
                    forecasts = kickoff(&world, *depth as u8, None, false, 2.0);
                } else {
                    let allegiance = world.initiative;
                    let (time_key, increment_key) = match allegiance {
                        Team::Orange => ("wtime", "winc"),
                        Team::Blue => ("btime", "binc")
                    };
                    let remaining_movements = options.get("movestogo")
                        .expect("expected movestogo");
                    let remaining_moments = options.get(time_key).expect(time_key);
                    let zero = 0; // XXX really? really??
                    let grace = options.get(increment_key).unwrap_or(&zero);
                    let deadline = ((remaining_moments /
                                     remaining_movements) + grace) / 1000;
                    let (beforecasts, depth) = iterative_deepening_kickoff(
                        &world, Duration::seconds(deadline as i64), false, 2.0);
                    forecasts = beforecasts;
                    println!("info depth {}", depth);
                }
                let movement = forecasts.swap_remove(0).0;
                println!("bestmove {}{}",
                         movement.patch.whence.to_algebraic(),
                         movement.patch.whither.to_algebraic());
                world = movement.tree;
            }
            s @ _ => { moral_panic!(format!("got unrecognized UCI command {:?}", s)) }
        }

    }
}
