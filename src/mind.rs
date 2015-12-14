use std::f32::{NEG_INFINITY, INFINITY};
use std::cmp::Ordering;
use std::mem;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use time;
use lru_cache::LruCache;

use identity::{Team, JobDescription, Agent};
use life::{Commit, Patch, WorldState};
use landmark::{CENTER_OF_THE_WORLD, LOW_COLONELCY,
               LOW_SEVENTH_HEAVEN, HIGH_COLONELCY, HIGH_SEVENTH_HEAVEN};
use space::Pinfield;
use substrate::Bytes;


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

    // ponies and servants want to be in the center of the world's action
    let center = Pinfield(CENTER_OF_THE_WORLD);
    // cast to signed to avoid overflow
    let orange_centerism: i8 = world.orange_servants.union(
        world.orange_ponies).intersection(center).pincount() as i8;
    let blue_centerism: i8 = world.blue_servants.union(
        world.blue_ponies).intersection(center).pincount() as i8;
    valuation += 0.1 * (orange_centerism - blue_centerism) as f32;

    // a cop's favorite beat is the seventh rank
    let high_seventh = Pinfield(HIGH_SEVENTH_HEAVEN);
    let orange_beat = world.orange_cops.intersection(high_seventh).pincount();
    valuation += 0.5 * orange_beat as f32;
    let low_seventh = Pinfield(LOW_SEVENTH_HEAVEN);
    let blue_beat = world.blue_cops.intersection(low_seventh).pincount();
    valuation -= 0.5 * blue_beat as f32;

    // servants should aspire to something more in life someday
    let orange_subascendants = world.orange_servants.intersection(high_seventh)
        .pincount();
    valuation += 1.8 * orange_subascendants as f32;
    let high_colonelcy = Pinfield(HIGH_COLONELCY);
    let orange_subsubascendants = world.orange_servants.intersection(
        high_colonelcy).pincount();
    valuation += 0.6 * orange_subsubascendants as f32;
    let blue_subascendants = world.blue_servants.intersection(low_seventh)
        .pincount();
    valuation -= 1.8 * blue_subascendants as f32;
    let low_colonelcy = Pinfield(LOW_COLONELCY);
    let blue_subsubascendants = world.blue_servants.intersection(
        low_colonelcy).pincount();
    valuation -= 0.6 * blue_subsubascendants as f32;

    // secret service eligbility has option value
    if world.orange_west_service_eligibility ||
        world.orange_east_service_eligibility {
            valuation += 0.1
    }
    if world.blue_west_service_eligibility ||
        world.blue_east_service_eligibility {
            valuation -= 0.1
    }

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

fn order_movements_heuristically(commits: &mut Vec<Commit>) {
    commits.sort_by(|a, b| {
        mmv_lva_heuristic(&b)
            .partial_cmp(&mmv_lva_heuristic(&a))
            .unwrap_or(Ordering::Equal)
    });
}

pub type Variation = Vec<Patch>;

#[derive(Debug, Clone)]
pub struct Lodestar {
    pub score: f32,
    pub variation: Variation
}

impl Lodestar {
    fn new(score: f32, variation: Variation) -> Self {
        Lodestar { score: score, variation: variation }
    }
}


pub fn α_β_negamax_search(
    world: WorldState, depth: u8, mut α: f32, β: f32, variation: Variation,
    memory_bank: Arc<Mutex<LruCache<WorldState, Lodestar>>>)
        -> Lodestar {
    let mut premonitions = world.reckless_lookahead();
    order_movements_heuristically(&mut premonitions);
    if depth == 0 || premonitions.is_empty() {
        let potential_score = orientation(world.initiative) * score(world);
        return Lodestar::new(potential_score, variation)
    };
    let mut optimum = NEG_INFINITY;
    let mut optimand = variation.clone();
    for premonition in premonitions.into_iter() {
        let mut value = NEG_INFINITY;  // can't hurt to be pessimistic
        let mut extended_variation = variation.clone();
        extended_variation.push(premonition.patch);
        let cached: bool;
        {
            let mut open_vault = memory_bank.lock().unwrap();
            let lodestar_maybe = open_vault.get_mut(&premonition.tree);
            match lodestar_maybe {
                Some(lodestar) => {
                    cached = true;
                    value = lodestar.score;
                }
                None => { cached = false; }
            };
        }

        if !cached {
            let lodestar = α_β_negamax_search(
                premonition.tree, depth - 1,
                -β, -α, extended_variation.clone(), memory_bank.clone()
            );
            value = -lodestar.score;
            extended_variation = lodestar.variation;
            memory_bank.lock().unwrap().insert(
                premonition.tree,
                Lodestar::new(value, extended_variation.clone())
            );
        }

        if value > optimum {
            optimum = value;
            optimand = extended_variation;
        }
        if value > α {
            α = value;
        }
        if α >= β {
            break;
        }
    }

    Lodestar::new(optimum, optimand)
}


pub fn déjà_vu_table_size_bound(gib: f32) -> usize {
    usize::from(Bytes::gibi(gib)) /
        (mem::size_of::<WorldState>() + mem::size_of::<Lodestar>())
}


#[allow(type_complexity)]
pub fn potentially_timebound_kickoff(
    world: &WorldState, depth: u8,
    nihilistically: bool,
    deadline_maybe: Option<time::Timespec>,
    order_movements: &Fn(&mut Vec<Commit>) -> (), déjà_vu_bound: f32)
        -> Option<Vec<(Commit, f32, Variation)>> {
    let déjà_vu_table: LruCache<WorldState, Lodestar> =
        LruCache::new(déjà_vu_table_size_bound(déjà_vu_bound));
    let memory_bank = Arc::new(Mutex::new(déjà_vu_table));
    let mut premonitions;
    if nihilistically {
        premonitions = world.reckless_lookahead();
    } else {
        premonitions = world.lookahead();
    }
    order_movements(&mut premonitions);
    let mut forecasts = Vec::new();
    let mut time_radios: Vec<(Commit, mpsc::Receiver<Lodestar>)> = Vec::new();
    for &premonition in &premonitions {
        let travel_bank = memory_bank.clone();
        let (tx, rx) = mpsc::channel();
        let explorer_radio = tx.clone();
        time_radios.push((premonition, rx));
        thread::spawn(move || {
            let search_hit: Lodestar = α_β_negamax_search(
                premonition.tree, depth - 1,
                NEG_INFINITY, INFINITY, vec![premonition.patch],
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
                let value = -search_hit.score;
                forecasts.push((premonition, value, search_hit.variation));
                time_radios.swap_remove(i);
            }
        }
        thread::sleep(Duration::from_millis(2));
    }
    forecasts.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
    Some(forecasts)
}


pub fn kickoff(world: &WorldState, depth: u8,
               nihilistically: bool, déjà_vu_bound: f32)
                   -> Vec<(Commit, f32, Variation)> {
    potentially_timebound_kickoff(world, depth, nihilistically, None,
                                  &order_movements_heuristically,
                                  déjà_vu_bound).unwrap()
}


fn inductive_movement_imposition(prophecy: &[(Commit, f32, Variation)])
                                 -> Box<Fn(&mut Vec<Commit>) -> ()> {
    let premonitions = prophecy.iter().map(|p| p.0).collect::<Vec<_>>();
    // SNEAKY: we expect to call the returned imposition with an argument whose
    // elements are the same commits that are the first elements of the tuples
    // that are the elements of `prophecy`—that's why it's OK to clobber them
    // like this
    Box::new(move |ps| { *ps = premonitions.clone(); })
}


pub fn iterative_deepening_kickoff(world: &WorldState, timeout: time::Duration,
                                   nihilistically: bool, déjà_vu_bound: f32)
                                   -> (Vec<(Commit, f32, Variation)>, u8) {
    let deadline = time::get_time() + timeout;
    let mut depth = 1;
    let mut forecasts = potentially_timebound_kickoff(
        world, depth, nihilistically, None,
        &order_movements_heuristically,
        déjà_vu_bound).unwrap();
    let mut order_movements = inductive_movement_imposition(&forecasts);
    while let Some(prophecy) = potentially_timebound_kickoff(
            world, depth, nihilistically, Some(deadline),
            &*order_movements, déjà_vu_bound) {
        order_movements = inductive_movement_imposition(&prophecy);
        forecasts = prophecy;
        depth += 1;
    }
    (forecasts, depth-1)
}


pub fn fixed_depth_sequence_kickoff(world: &WorldState, depth_sequence: Vec<u8>,
                                    nihilistically: bool, déjà_vu_bound: f32)
                                    -> Vec<(Commit, f32, Variation)> {
    let mut depths = depth_sequence.iter();
    let mut forecasts = potentially_timebound_kickoff(
        world, *depths.next().expect("`depth_sequence` should be nonempty"),
        nihilistically, None, &order_movements_heuristically,
        déjà_vu_bound
    ).unwrap();
    let mut order_movements = inductive_movement_imposition(&forecasts);
    for &depth in depths {
        forecasts = potentially_timebound_kickoff(
            world, depth, nihilistically, None,
            &*order_movements, déjà_vu_bound).unwrap();
        order_movements = inductive_movement_imposition(&forecasts);
    }
    forecasts
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

    const MOCK_DÉJÀ_VU_BOUND: f32 = 2.0;

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
        b.iter(|| kickoff(&ws, 1, true, MOCK_DÉJÀ_VU_BOUND));
    }

    #[bench]
    fn benchmark_kickoff_depth_2_arbys(b: &mut Bencher) {
        let ws = WorldState::new();
        b.iter(|| kickoff(&ws, 2, true, MOCK_DÉJÀ_VU_BOUND));
    }

    #[bench]
    fn benchmark_kickoff_depth_2_carefully(b: &mut Bencher) {
        let ws = WorldState::new();
        b.iter(|| kickoff(&ws, 2, false, MOCK_DÉJÀ_VU_BOUND));
    }

    #[bench]
    fn benchmark_kickoff_depth_3(b: &mut Bencher) {
        let ws = WorldState::new();
        b.iter(|| kickoff(&ws, 3, true, MOCK_DÉJÀ_VU_BOUND));
    }

    #[test]
    #[ignore]  // more research is needed
    fn concerning_short_circuiting_upon_finding_critical_endangerment() {
        let ws = WorldState::reconstruct("7K/r7/1r6/8/8/8/8/7k b -".to_owned());
        let start = time::get_time();
        kickoff(&ws, 30, true, MOCK_DÉJÀ_VU_BOUND);
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
        // looking ahead 3 movements allows the Leafline AI to catch the
        // split, whereby transforming into a pony (rather than
        // transitioning into a princess, as would usually be
        // expected) endangers both the blue princess and figurehead
        let tops = kickoff(&ws, 3, true, MOCK_DÉJÀ_VU_BOUND);
        let best_move = tops[0].0;
        let score = tops[0].1;
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
        let advisory = kickoff(&world, depth, true, MOCK_DÉJÀ_VU_BOUND);

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
        negaworld.initiative = Team::Blue;

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
        negaworld.initiative = Team::Blue;

        negaworld.no_castling_at_all();

        let negadvisory = kickoff(&negaworld, depth, true, MOCK_DÉJÀ_VU_BOUND);

        // taking the pony is still the right thing to do, even in the
        // negaworld
        assert_eq!(Locale { rank: 0, file: 0 },
                   negadvisory[0].0.patch.whither);
    }

}
