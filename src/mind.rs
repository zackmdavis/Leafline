use std::f32::{NEG_INFINITY, INFINITY};
use std::cmp::Ordering;
use std::collections::HashMap;

use identity::{Team, JobDescription, Agent};
use life::{Commit, WorldState};
use motion::CENTER_OF_THE_WORLD;
use space::Pinfield;


pub fn orientation(team: Team) -> f32 {
    match team {
        Team::Orange => 1.0,
        Team::Blue => -1.0,
    }
}

pub fn figurine_valuation(agent: Agent) -> f32 {
    let value = match agent.job_description {
        // en.wikipedia.org/wiki/
        // Chess_piece_relative_value#Hans_Berliner.27s_system
        JobDescription::Servant => 1.0,
        JobDescription::Pony => 3.2,
        JobDescription::Scholar => 3.3,
        JobDescription::Cop => 5.1,
        JobDescription::Princess => 8.8,
        JobDescription::Figurehead => 20000.0,
    };
    orientation(agent.team) * value
}

pub fn score(world: WorldState) -> f32 {
    let mut valuation = 0.0;

    for team in Team::league().into_iter() {
        for agent in Agent::dramatis_personæ(team).into_iter() {
            valuation += world.agent_to_pinfield_ref(
                agent).pincount() as f32 * figurine_valuation(agent);
        }
        // breadth of scholarship bonus
        if world.agent_to_pinfield_ref(Agent {
                    team: team,
                    job_description: JobDescription::Scholar,
                })
                .to_locales()
                .len() >= 2 {
            valuation += orientation(team) * 0.5
        }
    }

    // ponies and servants want to be in the center of the world's
    // action, maybe??
    let center = Pinfield(CENTER_OF_THE_WORLD);
    // cast to signed to avoid overflow
    let orange_centerism: i8 = world.orange_servants.union(
        world.orange_ponies).intersection(center).pincount() as i8;
    let blue_centerism: i8 = world.blue_servants.union(
        world.blue_ponies).intersection(center).pincount() as i8;
    valuation += (orange_centerism - blue_centerism) as f32 * 0.1;

    valuation
}

fn mmv_lva_heuristic(commit: &Commit) -> f32 {
    // https://chessprogramming.wikispaces.com/MVV-LVA
    match commit.hospitalization {
        Some(patient) => (
            figurine_valuation(patient) - figurine_valuation(commit.patch.star)),
        None => 0.0,
    }
}

fn order_moves(commits: &mut Vec<Commit>) {
    commits.sort_by(|a, b| {
        mmv_lva_heuristic(&b)
            .partial_cmp(&mmv_lva_heuristic(&a))
            .unwrap_or(Ordering::Equal)
    });
}


#[allow(dead_code)]  // somehow feel like leaving this checked in for now
pub fn negamax_search(world: WorldState, depth: u8) -> (Option<Commit>, f32) {
    let team = world.to_move;
    let mut premonitions = world.reckless_lookahead();
    order_moves(&mut premonitions);
    if depth == 0 || premonitions.is_empty() {
        return (None, orientation(team) * score(world))
    }
    let mut optimum = NEG_INFINITY;
    let mut optimand = None;
    for premonition in premonitions.into_iter() {
        let (_after, mut value) = negamax_search(premonition.tree, depth - 1);
        value = -value;
        if value > optimum {
            optimum = value;
            optimand = Some(premonition);
        }
    }
    (optimand, optimum)
}


pub fn α_β_negamax_search(world: WorldState,
                            depth: u8,
                            alpha: f32,
                            beta: f32,
                            deja_vu_table: &mut HashMap<WorldState, f32>)
                            -> (Option<Commit>, f32) {

    // RESEARCH: I don't really care that much right now, but can you
    // mutate (reassign) an argument name, and if so, what is the syntax?
    let mut experienced_alpha = alpha;
    let team = world.to_move;
    let mut premonitions = world.reckless_lookahead();
    order_moves(&mut premonitions);
    if depth == 0 || premonitions.is_empty() {
        return (None, orientation(team) * score(world))
    };
    let mut optimum = NEG_INFINITY;
    let mut optimand = None;
    for premonition in premonitions.into_iter() {
        let mut value: f32;
        let cached: bool;
        {
            let cached_value_maybe = deja_vu_table.get(&premonition.tree);
            match cached_value_maybe {
                Some(&cached_value) => {
                    cached = true;
                    value = cached_value;
                }
                None => {
                    cached = false;
                    // XXX fake assignment to work around the compiler's
                    // "possibly uninitialized value" rules
                    value = NEG_INFINITY;
                }
            };
        }

        if !cached {
            let (_, acquired_value) = α_β_negamax_search(premonition.tree,
                                                         depth - 1,
                                                         -beta,
                                                         -experienced_alpha,
                                                         deja_vu_table);
            value = -acquired_value;
            deja_vu_table.insert(premonition.tree, value);
        }

        if value > optimum {
            optimum = value;
            optimand = Some(premonition);
        }
        if value > experienced_alpha {
            experienced_alpha = value;
        }
        if experienced_alpha >= beta {
            break;
        }
    }

    (optimand, optimum)
}


// The vision here is that for the turn I'm immediately going to take,
// I want a report of all possible moves ranked by negamax-computed
// optimality, but that we don't want to bother with all that bookkeeping
// for subsequent levels of the game tree; minimax is expensive enough
// already!!
pub fn kickoff(world: &WorldState, depth: u8,
               nihilistically: bool) -> Vec<(Commit, f32)> {
    // when we get non-ASCII identifiers: `déjà_vu_table`
    let mut deja_vu_table: HashMap<WorldState, f32> = HashMap::new();
    let mut premonitions;
    if nihilistically {
        premonitions = world.reckless_lookahead();
    } else {
        premonitions = world.lookahead();
    }
    order_moves(&mut premonitions);
    let mut forecasts = Vec::new();
    for premonition in premonitions.into_iter() {
        let (_grandchild, mut value) = α_β_negamax_search(premonition.tree,
                                                          depth - 1,
                                                          NEG_INFINITY,
                                                          INFINITY,
                                                          &mut deja_vu_table);
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
mod tests {
    extern crate test;
    use self::test::Bencher;

    use super::{kickoff, score};
    use space::Locale;
    use life::WorldState;
    use identity::Team;

    impl WorldState {
        fn no_castling_at_all(&mut self) {
            self.orange_east_service_eligibility = false;
            self.orange_west_service_eligibility = false;
            self.blue_east_service_eligibility = false;
            self.blue_west_service_eligibility = false;
        }
    }


    #[bench]
    fn benchmark_scoring(b: &mut Bencher) {
        b.iter(|| score(WorldState::new()));
    }

    #[bench]
    fn benchmark_kickoff_depth_1(b: &mut Bencher) {
        let ws = WorldState::new();
        b.iter(|| kickoff(&ws, 1, true));
    }

    #[bench]
    fn benchmark_kickoff_depth_2_arbys(b: &mut Bencher) {
        let ws = WorldState::new();
        b.iter(|| kickoff(&ws, 2, true));
    }

    #[bench]
    fn benchmark_kickoff_depth_2_carefully(b: &mut Bencher) {
        let ws = WorldState::new();
        b.iter(|| kickoff(&ws, 2, false));
    }

    #[ignore]
    #[bench]
    fn benchmark_kickoff_depth_3(b: &mut Bencher) {
        let ws = WorldState::new();
        b.iter(|| kickoff(&ws, 3, true));
    }

    #[test]
    #[allow(float_cmp)]
    fn concerning_fairness_of_the_initial_position() {
        // its okay to assume this is *really* 0.0. floats may be imprecise,
        // but they do have well-defined behaviour.
        assert_eq!(0.0, score(WorldState::new()));
    }

    #[test]
    fn concerning_servant_ascension_choices() {
        let ws = WorldState::reconstruct("8/q1P1k/8/8/8/8/6PP/7K w -".to_owned());
        // looking ahead 3 moves allows leafline to catch the split
        let (ref best_move, score) = kickoff(&ws, 3, true)[0];
        println!("{:?}", best_move);
        assert!(score > 0.0);
        assert_eq!(best_move.tree.preserve(), "2N5/q3k/8/8/8/8/6PP/7K b -");
    }

    #[test]
    fn experimentally_about_kickoff() {
        let mut world = WorldState::new_except_empty();
        // SCENARIO: let's imagine Orange (to move) has separate attacks against
        // Blue's pony and servant, against which Blue has no defense but
        // to run away. We predict that Orange will take the pony, and
        // then Blue will move the servant out of the way.

        // scholar endangers pony
        world.blue_ponies = world.blue_ponies.alight(
            Locale { rank: 0, file: 0 });
        world.orange_scholars = world.orange_scholars.alight(
            Locale { rank: 2, file: 2 });

        // pony endangers servant
        world.blue_servants = world.blue_servants.alight(
            Locale { rank: 7, file: 1 });
        world.orange_ponies = world.orange_ponies.alight(
            Locale { rank: 5, file: 2 });

        // Blue has another servant sitting nowhere interesting
        world.blue_servants = world.blue_servants.alight(
            Locale { rank: 3, file: 6 });
        world.no_castling_at_all();

        let depth = 2;
        let advisory = kickoff(&world, depth, true);

        world.display();
        advisory[0].0.tree.display();
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
        negaworld.to_move = Team::Blue;

        // scholar endangers pony
        negaworld.orange_ponies = negaworld.orange_ponies.alight(
            Locale { rank: 0, file: 0 });
        negaworld.blue_scholars = negaworld.blue_scholars.alight(
            Locale { rank: 2, file: 2 });

        // pony endangers servant
        negaworld.orange_servants = negaworld.orange_servants.alight(
            Locale { rank: 7, file: 1 });
        negaworld.blue_ponies = negaworld.blue_ponies.alight(
            Locale { rank: 5, file: 2 });

        // Orange has another servant sitting nowhere interesting
        negaworld.orange_servants = negaworld.orange_servants.alight(
            Locale { rank: 3, file: 6 });
        negaworld.to_move = Team::Blue;

        negaworld.no_castling_at_all();

        let negadvisory = kickoff(&negaworld, depth, true);

        // taking the pony is still the right thing to do, even in the
        // negaworld
        assert_eq!(Locale { rank: 0, file: 0 },
                   negadvisory[0].0.patch.whither);
    }

}
