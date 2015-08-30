use std::f32::NEG_INFINITY;
use std::cmp::Ordering;

use identity::{Team, JobDescription, Agent};
use life::{Commit, WorldState};

pub fn orientation(team: Team) -> f32 {
    match team {
        Team::Orange => 1.0,
        Team::Blue => -1.0
    }
}

pub fn figurine_valuation(agent: Agent) -> f32 {
    let Agent { team: team, job_description: job_description } = agent;
    let value = match job_description {
        // en.wikipedia.org/wiki/
        // Chess_piece_relative_value#Hans_Berliner.27s_system
        JobDescription::Servant => 1.0,
        JobDescription::Pony => 3.2,
        JobDescription::Scholar => 3.3,
        JobDescription::Cop => 5.1,
        JobDescription::Princess => 8.8,
        JobDescription::Figurehead => 20000.0
    };
    orientation(team) * value
}

pub fn score(world: WorldState) -> f32 {
    let mut valuation = 0.0;
    for team in Team::league().into_iter() {
        for agent in Agent::dramatis_personae(team).into_iter() {
            valuation += world.agent_to_pinfield_ref(
                agent).pincount() as f32 * figurine_valuation(agent);
        }
    }
    valuation
}

pub fn negamax_search(world: WorldState, depth: u8) -> (Option<Commit>, f32) {
    let team = world.to_move;
    let premonitions = world.reckless_lookahead();
    if depth == 0 || premonitions.is_empty() {
        return (None, orientation(team) * score(world))
    }
    let mut optimum = NEG_INFINITY;
    let mut optimand = None;
    for premonition in premonitions.into_iter() {
        let (_after, mut value) = negamax_search(premonition.tree, depth-1);
        value = -value;
        if value > optimum {
            optimum = value;
            optimand = Some(premonition);
        }
    }
    (optimand, optimum)
}


// The vision here is that for the turn I'm immediately going to take,
// I want a report of all possible moves ranked by negamax-computed
// optimality, but that we don't want to bother with all that bookkeeping
// for subsequent levels of the game tree; minimax is expensive enough
// already!!
pub fn negamax_kickoff(world: WorldState, depth: u8) -> Vec<(Commit, f32)> {
    let team = world.to_move;
    let premonitions = world.lookahead();
    let mut forecasts = Vec::new();
    for premonition in premonitions.into_iter() {
        let (_grandchild, mut value) = negamax_search(premonition.tree, depth-1);
        value = -value;
        forecasts.push((premonition, value));
    }
    // The circumlocution (thanks to
    // https://www.reddit.com/r/rust/comments/29kia3/no_ord_for_f32/ for
    // the suggestion) is because Rust is sanctimonious about IEEE 754
    // floats not being totally ordered
    forecasts.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
    forecasts
}


#[cfg(test)]
mod test {
    use time;

    use super::{negamax_search, negamax_kickoff, score};
    use space::Locale;
    use life::WorldState;

    #[test]
    fn concerning_fairness_of_the_initial_position() {
        assert_eq!(0.0, score(WorldState::new()));
    }

    #[test]
    fn experimentally_about_negamax_kickoff() {
        let mut world = WorldState::new_except_empty();
        // SCENARIO: let's imagine Orange (to move) has separate attacks against
        // Blue's pony and servant, against which Blue has no defense but
        // to run away. We predict that Orange will take the pony, and
        // then Blue will move the servant out of the way.

        // scholar endangers pony
        world.blue_ponies = world.blue_ponies.alight(
            Locale { rank: 0, file: 0 }
        );
        world.orange_scholars = world.orange_scholars.alight(
            Locale { rank: 2, file: 2 }
        );

        // pony endangers servant
        world.blue_servants = world.blue_servants.alight(
            Locale { rank: 7, file: 1 }
        );
        world.orange_ponies = world.orange_ponies.alight(
            Locale { rank: 5, file: 2 }
        );

        // Blue has another servant sitting nowhere interesting
        world.blue_servants = world.blue_servants.alight(
            Locale { rank: 3, file: 6 }
        );

        let depth = 2;
        let start = time::get_time();
        let advisory = negamax_kickoff(world, depth);
        let end = time:: get_time();

        // (you can see this if you run the tests with `-- --nocapture`)
        println!("negamax kickoff: evaluating {} possible choices to \
                  depth {} took {:?}", advisory.len(), depth, end-start);
        for item in advisory.iter() {
            println!("{:?}", item);
        }

        // taking the pony is the right thing to do
        assert_eq!(Locale { rank: 0, file: 0 }, advisory[0].0.patch.whither);

        // And, furthermore, the answer should be the same if we face the
        // same situation with the colors reversed
        //
        // XXX this would be tidier and less copy-pastey if I had more
        // general figurine-placing functions that were three rather than
        // two levels of abstraction above twiddling bits on an unsigned
        // int ... oh, well
        let mut negaworld = WorldState::new_except_empty();

        // scholar endangers pony
        negaworld.orange_ponies = negaworld.orange_ponies.alight(
            Locale { rank: 0, file: 0 }
        );
        negaworld.blue_scholars = negaworld.blue_scholars.alight(
            Locale { rank: 2, file: 2 }
        );

        // pony endanger servant
        negaworld.orange_servants = negaworld.orange_servants.alight(
            Locale { rank: 7, file: 1 }
        );
        negaworld.blue_ponies = negaworld.blue_ponies.alight(
            Locale { rank: 5, file: 2 }
        );

        // Orange has another servant sitting nowhere interesting
        negaworld.orange_servants = world.orange_servants.alight(
            Locale { rank: 3, file: 6 }
        );

        let negadvisory = negamax_kickoff(negaworld, depth);

        // taking the pony is still the right thing to do, even in the
        // negaworld
        assert_eq!(Locale { rank: 0, file: 0 }, advisory[0].0.patch.whither);
    }

}
