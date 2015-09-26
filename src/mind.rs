use std::f32::{NEG_INFINITY, INFINITY};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::thread;

use time;

use identity::{Team, JobDescription, Agent};
use life::{Commit, WorldState};
use landmark::CENTER_OF_THE_WORLD;
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

fn order_moves(commits: &mut Vec<Commit>,
               déjà_vu_table: &HashMap<WorldState, f32>) {
    commits.sort_by(|a, b| {
        déjà_vu_table.get(&b.tree).unwrap_or(&NEG_INFINITY)
            .partial_cmp(&déjà_vu_table.get(&a.tree).unwrap_or(&NEG_INFINITY))
            .unwrap_or(Ordering::Equal)
    });
    commits.sort_by(|a, b| {  // prioritize favorable-looking exchanges ...
        mmv_lva_heuristic(&b)
            .partial_cmp(&mmv_lva_heuristic(&a))
            .unwrap_or(Ordering::Equal)  // (NaN is a non-issue)
    });
    // // but hit the cache first (`.sort_by` is stable)
    // commits.sort_by(|a, b| {
    //     déjà_vu_table.get(&b.tree).unwrap_or(&NEG_INFINITY)
    //         .partial_cmp(&déjà_vu_table.get(&a.tree).unwrap_or(&NEG_INFINITY))
    //         .unwrap_or(Ordering::Equal)
    // });
}

pub fn α_β_negamax_search(
    world: WorldState, depth: u8, mut α: f32, β: f32,
    memory_bank: Arc<Mutex<HashMap<WorldState, f32>>>) -> (Option<Commit>, f32) {
    let team = world.to_move;
    let potential_score = orientation(team) * score(world);
    let mut premonitions = world.reckless_lookahead();
    {
        let open_vault = memory_bank.lock().unwrap();
        order_moves(&mut premonitions, &open_vault);
    }
    if depth == 0 || premonitions.is_empty() {
        return (None, potential_score)
    };
    let mut optimum = NEG_INFINITY;
    let mut optimand = None;
    for premonition in premonitions.into_iter() {
        let mut value = NEG_INFINITY;  // can't hurt to be pessimistic
        let cached: bool;
        {
            let open_vault = memory_bank.lock().unwrap();
            let cached_value_maybe = open_vault.get(&premonition.tree);
            match cached_value_maybe {
                Some(&cached_value) => {
                    cached = true;
                    value = cached_value;
                }
                None => { cached = false; }
            };
        }

        if !cached {
            let (_, acquired_value) = α_β_negamax_search(
                premonition.tree, depth - 1,
                -β, -α, memory_bank.clone()
            );
            value = -acquired_value;
            memory_bank.lock().unwrap().insert(premonition.tree, value);
        }

        if value > optimum {
            optimum = value;
            optimand = Some(premonition);
        }
        if value > α {
            α = value;
        }
        if α >= β {
            break;
        }
    }

    (optimand, optimum)
}

#[allow(type_complexity)]
pub fn potentially_timebound_kickoff(world: &WorldState, depth: u8,
                                     nihilistically: bool,
                                     deadline_maybe: Option<time::Timespec>)
                                     -> Option<Vec<(Commit, f32)>> {
    let déjà_vu_table: HashMap<WorldState, f32> = HashMap::new();
    let memory_bank = Arc::new(Mutex::new(déjà_vu_table));
    let mut premonitions;
    if nihilistically {
        premonitions = world.reckless_lookahead();
    } else {
        premonitions = world.lookahead();
    }
    {
        let open_vault = memory_bank.lock().unwrap();
        order_moves(&mut premonitions, &open_vault);
    }
    let mut forecasts = Vec::new();
    let mut time_radios: Vec<(Commit, mpsc::Receiver<(Option<Commit>, f32)>)> =
        Vec::new();
    for &premonition in &premonitions {
        let travel_bank = memory_bank.clone();
        let (tx, rx) = mpsc::channel();
        let explorer_radio = tx.clone();
        time_radios.push((premonition, rx));
        thread::spawn(move || {
            let search_hit: (Option<Commit>, f32) = α_β_negamax_search(
                premonition.tree, depth - 1,
                NEG_INFINITY, INFINITY,
                travel_bank
            );
            explorer_radio.send(search_hit).ok();
        });
    }
    while !time_radios.is_empty() {  // polling for results
        if let Some(deadline) = deadline_maybe {
            if time::get_time() > deadline {
                return None;
            }
        }
        // iterate over indices so that we can use swap_remove during the loop
        for i in (0..time_radios.len()).rev() {
            let premonition = time_radios[i].0;
            if let Some(search_hit) = time_radios[i].1.try_recv().ok() {
                let (_grandchild, mut value) = search_hit;
                value = -value;
                forecasts.push((premonition, value));
                time_radios.swap_remove(i);
            }
        }
        thread::sleep_ms(2);
    }
    forecasts.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
    Some(forecasts)
}


pub fn kickoff(world: &WorldState, depth: u8,
               nihilistically: bool) -> Vec<(Commit, f32)> {
    potentially_timebound_kickoff(world, depth, nihilistically, None).unwrap()
}


pub fn iterative_deepening_kickoff(world: &WorldState, timeout: time::Duration,
                                   nihilistically: bool)
                                   -> (Vec<(Commit, f32)>, u8) {
    let deadline = time::get_time() + timeout;
    let mut depth = 1;
    let mut forecasts = potentially_timebound_kickoff(
        world, depth, nihilistically, None).unwrap();
    while let Some(prophecy) = potentially_timebound_kickoff(
            world, depth, nihilistically, Some(deadline)) {
        forecasts = prophecy;
        depth += 1;
    }
    (forecasts, depth-1)
}


#[cfg(test)]
mod tests {
    extern crate test;
    use self::test::Bencher;

    use time;
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

    #[bench]
    fn benchmark_kickoff_depth_3(b: &mut Bencher) {
        let ws = WorldState::new();
        b.iter(|| kickoff(&ws, 3, true));
    }

    #[test]
    #[ignore]  // more research is needed
    fn concerning_short_circuiting_upon_finding_critical_endangerment() {
        let ws = WorldState::reconstruct("7K/r7/1r6/8/8/8/8/7k b -".to_owned());
        let start = time::get_time();
        kickoff(&ws, 30, true);
        let duration = time::get_time() - start;
        assert!(duration.num_seconds() < 20);
    }

    #[test]
    #[allow(float_cmp)]
    fn concerning_fairness_of_the_initial_position() {
        // It's okay to assume this is really 0.0. Floats may be imprecise,
        // but they do have well-defined behavior.
        assert_eq!(0.0, score(WorldState::new()));
    }

    #[test]
    fn concerning_servant_ascension_choices() {
        let ws = WorldState::reconstruct("8/q1P1k/8/8/8/8/6PP/7K w -".to_owned());
        // looking ahead 3 moves allows the Leafline AI to catch the
        // split, whereby transforming into a pony (rather than
        // transitioning into a princess, as would usually be
        // expected) endangers both the blue princess and figurehead
        let (ref best_move, score) = kickoff(&ws, 3, true)[0];
        println!("{:?}", best_move);
        assert!(score > 0.0);
        assert_eq!(best_move.tree.preserve(), "2N5/q3k3/8/8/8/8/6PP/7K b -");
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
