use std::io;
use std::io::Write;

use mind::kickoff;
use life::WorldState;
use space::Locale;


// Unlikely Command Integration dæmon

pub fn dæmon(depth: u8) {
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
                // we should at least be able to support "depth" and "nodes"
                // options from the game-driver but don't worry about that now;
                // just search at the depth we want to
                let mut forecasts = kickoff(&world, depth, None, false, 2.0);
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
