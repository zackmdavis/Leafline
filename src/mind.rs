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
    fn experimentally_about_negamax_kickoff_in_faceoff_while_masters_away() {
        let mut world = WorldState::new_except_empty();
        let orange_files = [1u8, 2, 3];
        let blue_files = [2u8, 3, 4];
        for &file in orange_files.iter() {
            world.orange_servants = world.orange_servants.alight(
                Locale { rank: 3, file: file }
            );
        }
        for &file in blue_files.iter() {
            world.blue_servants = world.blue_servants.alight(
                Locale { rank: 4, file: file }
            );
        }
        println!("negamax kickoff {:?}", negamax_kickoff(world, 2));
        // TODO: learn how to debug stuff
    }

}
