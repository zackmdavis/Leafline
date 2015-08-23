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
    let premonitions = world.lookahead();
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
        let (_grandchild, score) = negamax_search(premonition.tree, depth-1);
        forecasts.push((premonition, score));
    }
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
    #[ignore]
    fn concerning_negamax_implementation_suitability() {
        for depth in 1..5 {
            let start = time::get_time();
            negamax_search(WorldState::new(), depth);
            let end = time::get_time();
            println!("searching the initial position to depth {} took {:?}",
                     depth, end-start);
        }
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

        // pony endanger servant
        world.blue_servants = world.blue_servants.alight(
            Locale { rank: 7, file: 1 }
        );
        world.orange_ponies = world.orange_ponies.alight(
            Locale { rank: 5, file: 2 }
        );

        let depth = 2;
        let start = time::get_time();
        let advisory = negamax_kickoff(world, depth);
        let end = time:: get_time();

        println!("negamax kickoff: evaluating {} possible choices to \
                  depth {} took {:?}", advisory.len(), depth, end-start);
        for item in advisory.iter() {
            println!("{:?}", item);
        }
        // XXX OK, I think I must have a sign error somewhere;
        // capturing the servant and pony are the lowest-ranked options
        // at -3.3 and -5.5, respectively: and the top option is to move
        // the Orange pony to a7 where the servant can get it!
    }

}
