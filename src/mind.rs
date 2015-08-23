use std::f32::NEG_INFINITY;

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


#[cfg(test)]
mod test {
    use time;

    use super::{negamax_search, score};
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
}
