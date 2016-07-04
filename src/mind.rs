use std::f32::{INFINITY, NEG_INFINITY};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::default::Default;
use std::fmt;
use std::hash::BuildHasherDefault;
use std::mem;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use time;
use lru_cache::LruCache;
use parking_lot;
use twox_hash::XxHash;

use identity::{Agent, JobDescription, Team};
use life::{Commit, Patch, WorldState};
use landmark::{CENTER_OF_THE_WORLD, HIGH_COLONELCY, HIGH_SEVENTH_HEAVEN,
               LOW_COLONELCY, LOW_SEVENTH_HEAVEN};
use space::Pinfield;
use substrate::Bytes;


const REWARD_FOR_INITIATIVE: f32 = 0.5;


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

    valuation += REWARD_FOR_INITIATIVE * orientation(world.initiative);

    for team in Team::league() {
        for agent in Agent::dramatis_personæ(team) {
            valuation += world.agent_to_pinfield_ref(agent)
                              .pincount() as f32 *
                         figurine_valuation(agent);
        }
        // breadth of scholarship bonus
        if world.agent_to_pinfield_ref(Agent {
                    team: team,
                    job_description: JobDescription::Scholar,
                }).pincount() >= 2 {
            valuation += orientation(team) * 0.5
        }
    }

    // ponies and servants want to be in the center of the world's action
    let center = Pinfield(CENTER_OF_THE_WORLD);
    // cast to signed to avoid overflow
    let orange_centerism: i8 = world.orange_servants
                                    .union(world.orange_ponies)
                                    .intersection(center)
                                    .pincount() as i8;
    let blue_centerism: i8 = world.blue_servants
                                  .union(world.blue_ponies)
                                  .intersection(center)
                                  .pincount() as i8;
    valuation += 0.1 * (orange_centerism - blue_centerism) as f32;

    // a cop's favorite beat is the seventh rank
    let high_seventh = Pinfield(HIGH_SEVENTH_HEAVEN);
    let orange_beat = world.orange_cops.intersection(high_seventh).pincount();
    valuation += 0.5 * orange_beat as f32;
    let low_seventh = Pinfield(LOW_SEVENTH_HEAVEN);
    let blue_beat = world.blue_cops.intersection(low_seventh).pincount();
    valuation -= 0.5 * blue_beat as f32;

    // servants should aspire to something more in life someday
    let orange_subascendants = world.orange_servants
                                    .intersection(high_seventh)
                                    .pincount();
    valuation += 1.8 * orange_subascendants as f32;
    let high_colonelcy = Pinfield(HIGH_COLONELCY);
    let orange_subsubascendants = world.orange_servants
                                       .intersection(high_colonelcy)
                                       .pincount();
    valuation += 0.6 * orange_subsubascendants as f32;
    let blue_subascendants = world.blue_servants
                                  .intersection(low_seventh)
                                  .pincount();
    valuation -= 1.8 * blue_subascendants as f32;
    let low_colonelcy = Pinfield(LOW_COLONELCY);
    let blue_subsubascendants = world.blue_servants
                                     .intersection(low_colonelcy)
                                     .pincount();
    valuation -= 0.6 * blue_subsubascendants as f32;

    // secret service eligbility has option value
    if world.orange_west_service_eligibility() ||
       world.orange_east_service_eligibility() {
        valuation += 0.1
    }
    if world.blue_west_service_eligibility() || world.blue_east_service_eligibility() {
        valuation -= 0.1
    }

    valuation
}

fn mmv_lva_heuristic(commit: &Commit) -> f32 {
    // https://chessprogramming.wikispaces.com/MVV-LVA
    match commit.hospitalization {
        Some(patient) => {
            (figurine_valuation(patient) - figurine_valuation(commit.patch.star))
        }
        None => 0.0,
    }
}

fn order_movements_heuristically(commits: &mut Vec<Commit>) {
    commits.sort_by(|a, b| {
        mmv_lva_heuristic(b)
            .partial_cmp(&mmv_lva_heuristic(a))
            .unwrap_or(Ordering::Equal)
    });
}

fn order_movements_intuitively(experience: &HashMap<Patch, u32>,
    commits: &mut Vec<Commit>) {
    commits.sort_by(|a, b| {
        let a_feels = experience.get(&a.patch);
        let b_feels = experience.get(&b.patch);
        b_feels.cmp(&a_feels)
    });
}

pub type Variation = Vec<Patch>;

#[allow(ptr_arg)]
pub fn pagan_variation_format(variation: &Variation) -> String {
    variation.iter()
             .map(|p| p.abbreviated_pagan_movement_rune())
             .collect::<Vec<_>>()
             .join(" ")
}


#[derive(Clone)]
pub struct Lodestar {
    pub score: f32,
    pub variation: Variation,
}

impl Lodestar {
    fn new(score: f32, variation: Variation) -> Self {
        Lodestar {
            score: score,
            variation: variation,
        }
    }
}

impl fmt::Debug for Lodestar {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f,
               "Lodestar {{ score: {}, variation: {} }}",
               self.score,
               pagan_variation_format(&self.variation))
    }
}


#[derive(Debug, Clone)]
pub struct Souvenir {
    soundness: u8,
    lodestar: Lodestar,
}

impl Souvenir {
    fn new(lodestar: Lodestar, field_depth: u8) -> Self {
        let soundness = lodestar.variation.len() as u8 - field_depth;
        Souvenir { soundness: soundness, lodestar: lodestar }
    }
}


#[allow(too_many_arguments)]
pub fn α_β_negamax_search(
    world: WorldState, depth: i8, mut α: f32, β: f32, variation: Variation,
    memory_bank: Arc<parking_lot::Mutex<LruCache<WorldState, Souvenir,
                                    BuildHasherDefault<XxHash>>>>,
    intuition_bank: Arc<parking_lot::Mutex<HashMap<Patch, u32>>>,
    quiet: Option<u8>)
        -> Lodestar {

    let mut premonitions = world.reckless_lookahead();
    let mut optimum = NEG_INFINITY;
    let mut optimand = variation.clone();
    if depth <= 0 || premonitions.is_empty() {
        let potential_score = orientation(world.initiative) * score(world);
        match quiet {
            None => {
                return Lodestar::new(potential_score, variation);
            },
            Some(extension) => {
                if depth.abs() >= extension as i8 {
                    return Lodestar::new(potential_score, variation);
                }
                premonitions = premonitions.into_iter()
                    .filter(|c| c.hospitalization.is_some())
                    .collect::<Vec<_>>();
                if premonitions.is_empty() {
                    return Lodestar::new(potential_score, variation)
                } else {
                    optimum = potential_score;
                }
            }
        }
    };

    order_movements_heuristically(&mut premonitions);
    {
        let experience = intuition_bank.lock();
        order_movements_intuitively(&experience, &mut premonitions)
    }
    for premonition in premonitions {
        let mut value = NEG_INFINITY;  // can't hurt to be pessimistic
        let mut extended_variation = variation.clone();
        extended_variation.push(premonition.patch);
        let cached: bool;
        {
            let mut open_vault = memory_bank.lock();
            let souvenir_maybe = open_vault.get_mut(&premonition.tree);
            match souvenir_maybe {
                Some(souvenir) => {
                    if souvenir.soundness as i8 >= depth {
                        cached = true;
                        value = souvenir.lodestar.score;
                        extended_variation = souvenir.lodestar.variation.clone();
                    } else {
                        cached = false;
                    }
                }
                None => { cached = false; }
            };
        }

        if !cached {
            let mut lodestar = α_β_negamax_search(
                premonition.tree, depth - 1,
                -β, -α, extended_variation.clone(),
                memory_bank.clone(), intuition_bank.clone(),
                quiet
            );
            lodestar.score *= -1.;  // nega-
            value = lodestar.score;
            extended_variation = lodestar.variation.clone();
            memory_bank.lock().insert(
                premonition.tree,
                Souvenir::new(lodestar, extended_variation.len() as u8)
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
            if depth > 0 { // not a quietness extension
                let mut open_vault = intuition_bank.lock();
                let mut intuition = open_vault.entry(premonition.patch)
                    .or_insert(0);
                *intuition += 2u32.pow(depth as u32);
            }
            break;  // cutoff!
        }
    }
    Lodestar::new(optimum, optimand)
}


pub fn déjà_vu_table_size_bound(gib: f32) -> usize {
    usize::from(Bytes::gibi(gib)) /
        (mem::size_of::<WorldState>() + mem::size_of::<Lodestar>())
}


pub fn potentially_timebound_kickoff(
    world: &WorldState, depth: u8,
    extension_maybe: Option<u8>,
    nihilistically: bool,
    deadline_maybe: Option<time::Timespec>,
    intuition_bank: Arc<parking_lot::Mutex<HashMap<Patch, u32>>>,
    déjà_vu_bound: f32)
        -> Option<Vec<(Commit, f32, Variation)>> {
    let déjà_vu_table: LruCache<WorldState, Souvenir,
                                BuildHasherDefault<XxHash>> =
        LruCache::with_hash_state(déjà_vu_table_size_bound(déjà_vu_bound),
                                  Default::default());
    let memory_bank = Arc::new(parking_lot::Mutex::new(déjà_vu_table));
    let mut premonitions = if nihilistically {
        world.reckless_lookahead()
    } else {
        world.lookahead()
    };
    order_movements_heuristically(&mut premonitions);
    {
        let experience = intuition_bank.lock();
        order_movements_intuitively(&experience, &mut premonitions)
    }
    let mut forecasts = Vec::new();
    let mut time_radios: Vec<(Commit, mpsc::Receiver<Lodestar>)> = Vec::new();
    for &premonition in &premonitions {
        let travel_memory_bank = memory_bank.clone();
        let travel_intuition_bank = intuition_bank.clone();
        let (tx, rx) = mpsc::channel();
        let explorer_radio = tx.clone();
        time_radios.push((premonition, rx));
        thread::spawn(move || {
            let search_hit: Lodestar = α_β_negamax_search(
                premonition.tree, (depth - 1) as i8,
                NEG_INFINITY, INFINITY, vec![premonition.patch],
                travel_memory_bank, travel_intuition_bank,
                extension_maybe
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
        debug!("waiting for {} of {} first-movement search threads",
               time_radios.len(), premonitions.len())
    }
    forecasts.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
    Some(forecasts)
}


pub fn kickoff(world: &WorldState, depth: u8, extension: Option<u8>,
               nihilistically: bool, déjà_vu_bound: f32)
                   -> Vec<(Commit, f32, Variation)> {
    let experience_table: HashMap<Patch, u32> = HashMap::new();
    let intuition_bank = Arc::new(parking_lot::Mutex::new(experience_table));
    potentially_timebound_kickoff(world, depth, extension, nihilistically, None,
                                  intuition_bank, déjà_vu_bound).unwrap()
}


pub fn iterative_deepening_kickoff(world: &WorldState, timeout: time::Duration,
                                   nihilistically: bool, déjà_vu_bound: f32)
                                   -> (Vec<(Commit, f32, Variation)>, u8) {
    let deadline = time::get_time() + timeout;
    let mut depth = 1;
    let experience_table: HashMap<Patch, u32> = HashMap::new();
    let intuition_bank = Arc::new(parking_lot::Mutex::new(experience_table));
    let mut forecasts = potentially_timebound_kickoff(
        world, depth, None, nihilistically, None,
        intuition_bank.clone(),
        déjà_vu_bound).unwrap();
    while let Some(prophecy) = potentially_timebound_kickoff(
            world, depth, None, nihilistically, Some(deadline),
            intuition_bank.clone(), déjà_vu_bound) {
        forecasts = prophecy;
        depth += 1;
    }
    (forecasts, depth-1)
}


pub fn fixed_depth_sequence_kickoff(world: &WorldState, depth_sequence: Vec<u8>,
                                    nihilistically: bool, déjà_vu_bound: f32)
                                    -> Vec<(Commit, f32, Variation)> {
    let mut depths = depth_sequence.iter();
    let experience_table: HashMap<Patch, u32> = HashMap::new();
    let intuition_bank = Arc::new(parking_lot::Mutex::new(experience_table));
    let mut forecasts = potentially_timebound_kickoff(
        world, *depths.next().expect("`depth_sequence` should be nonempty"),
        None, nihilistically, None, intuition_bank.clone(),
        déjà_vu_bound
    ).unwrap();
    for &depth in depths {
        forecasts = potentially_timebound_kickoff(
            world, depth, None, nihilistically, None,
            intuition_bank.clone(), déjà_vu_bound).unwrap();
    }
    forecasts
}


#[cfg(test)]
mod tests {
    extern crate test;
    use self::test::Bencher;

    use time;
    use super::{REWARD_FOR_INITIATIVE, kickoff, score};
    use space::Locale;
    use life::WorldState;
    use identity::Team;

    const MOCK_DÉJÀ_VU_BOUND: f32 = 2.0;

    impl WorldState {
        fn no_castling_at_all(&mut self) {
            self.clear_orange_east_service_eligibility();
            self.clear_orange_west_service_eligibility();
            self.clear_blue_east_service_eligibility();
            self.clear_blue_west_service_eligibility();
        }
    }

    #[bench]
    fn benchmark_scoring(b: &mut Bencher) {
        b.iter(|| score(WorldState::new()));
    }

    #[bench]
    fn benchmark_kickoff_depth_1(b: &mut Bencher) {
        let ws = WorldState::new();
        b.iter(|| kickoff(&ws, 1, None, true, MOCK_DÉJÀ_VU_BOUND));
    }

    #[bench]
    fn benchmark_kickoff_depth_2_arbys(b: &mut Bencher) {
        let ws = WorldState::new();
        b.iter(|| kickoff(&ws, 2, None, true, MOCK_DÉJÀ_VU_BOUND));
    }

    #[bench]
    fn benchmark_kickoff_depth_2_carefully(b: &mut Bencher) {
        let ws = WorldState::new();
        b.iter(|| kickoff(&ws, 2, None, false, MOCK_DÉJÀ_VU_BOUND));
    }

    #[bench]
    fn benchmark_kickoff_depth_3(b: &mut Bencher) {
        let ws = WorldState::new();
        b.iter(|| kickoff(&ws, 3, None, true, MOCK_DÉJÀ_VU_BOUND));
    }

    #[test]
    #[ignore]  // more research is needed
    fn concerning_short_circuiting_upon_finding_critical_endangerment() {
        let ws = WorldState::reconstruct("7K/r7/1r6/8/8/8/8/7k b -".to_owned());
        let start = time::get_time();
        kickoff(&ws, 30, None, true, MOCK_DÉJÀ_VU_BOUND);
        let duration = time::get_time() - start;
        assert!(duration.num_seconds() < 20);
    }

    #[test]
    #[allow(float_cmp)]
    fn concerning_fairness_of_the_initial_position() {
        // It's okay to assume this is really 0.0. Floats may be imprecise,
        // but they do have well-defined behavior.
        assert_eq!(0.0, score(WorldState::new()) - REWARD_FOR_INITIATIVE);
    }

    #[test]
    fn concerning_servant_ascension_choices() {
        let ws = WorldState::reconstruct("8/q1P1k/8/8/8/8/6PP/7K w - -".to_owned());
        // looking ahead 3 movements allows the Leafline AI to catch the
        // split, whereby transforming into a pony (rather than
        // transitioning into a princess, as would usually be
        // expected) endangers both the blue princess and figurehead
        let tops = kickoff(&ws, 3, None, true, MOCK_DÉJÀ_VU_BOUND);
        let best_move = tops[0].0;
        let score = tops[0].1;
        println!("{:?}", best_move);
        assert!(score > 0.0);
        assert_eq!(best_move.tree.preserve(), "2N5/q3k3/8/8/8/8/6PP/7K b - -");
    }

    #[test]
    fn experimentally_about_kickoff() {
        let mut world = WorldState::new_except_empty();
        // SCENARIO: let's imagine Orange (to move) has separate attacks against
        // Blue's pony and servant, against which Blue has no defense but
        // to run away. We predict that Orange will take the pony, and
        // then Blue will move the servant out of the way.

        // scholar endangers pony
        world.blue_ponies = world.blue_ponies.alight(Locale::new(0, 0));
        world.orange_scholars = world.orange_scholars.alight(Locale::new(2, 2));

        // pony endangers servant
        world.blue_servants = world.blue_servants.alight(Locale::new(7, 1));
        world.orange_ponies = world.orange_ponies.alight(Locale::new(5, 2));

        // Blue has another servant sitting nowhere interesting
        world.blue_servants = world.blue_servants.alight(Locale::new(3, 6));
        world.no_castling_at_all();

        let depth = 2;
        let advisory = kickoff(&world, depth, None, true, MOCK_DÉJÀ_VU_BOUND);

        // taking the pony is the right thing to do
        assert_eq!(Locale::new(0, 0), advisory[0].0.patch.whither);

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
        negaworld.orange_ponies = negaworld.orange_ponies.alight(Locale::new(0, 0));
        negaworld.blue_scholars = negaworld.blue_scholars.alight(Locale::new(2, 2));

        // pony endangers servant
        negaworld.orange_servants = negaworld.orange_servants
                                             .alight(Locale::new(7, 1));
        negaworld.blue_ponies = negaworld.blue_ponies.alight(Locale::new(5, 2));

        // Orange has another servant sitting nowhere interesting
        negaworld.orange_servants = negaworld.orange_servants
                                             .alight(Locale::new(3, 6));
        negaworld.initiative = Team::Blue;

        negaworld.no_castling_at_all();

        let negadvisory = kickoff(&negaworld, depth, None, true, MOCK_DÉJÀ_VU_BOUND);

        // taking the pony is still the right thing to do, even in the
        // negaworld
        assert_eq!(Locale::new(0, 0), negadvisory[0].0.patch.whither);
    }

    #[ignore]  // really slow
    #[test]
    fn concerning_fortune_favoring_the_bold() {
        // It would be nice if scores at even and odd plies were comparable,
        // rather than lurching wildly with parity due to the tempo
        // difference. We can try to compensate for this by accounting for
        // initiative in scoring world-states, but it's important to have a
        // test to demonstrate that the magnitude of that correction is
        // sane. ... although this is actually a pretty subtle problem where we
        // should be wary of making things worse.
        //
        // some "representative" scenarios ...
        let world_runesets = vec![
            // initial position
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            // 1. e4
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
            // princess's gambit declined I
            "rnbqkbnr/ppp2ppp/4p3/3p4/2PP4/8/PP2PPPP/RNBQKBNR w KQkq - 0 3",
            // princess's gambit declined II
            "rnbqkbnr/ppp2ppp/4p3/3p4/2PP4/2N5/PP2PPPP/R1BQKBNR b KQkq - 1 3",
            // powder keg I
            "r2q1rk1/2p1ppbp/1p4p1/p2p1b2/NnPPn3/PP2PN1P/1B2BPP1/R2Q1RK1 b - - 0 14",
            // powder keg II
            "r2q1rk1/2p1ppbp/1pn3p1/p2p1b2/N1PPn3/PP2PN1P/1B2BPP1/R2Q1RK1 w - - 1 15"
        ];
        let mut tempo_lurches: Vec<f32> = Vec::new();
        for world_runeset in world_runesets {
            let world = WorldState::reconstruct(world_runeset.to_owned());
            let mut previously = None;
            for &depth in &[2, 3, 4] {
                let premonitions = kickoff(&world, depth, None, true, 1.0);
                let mut top_showings = 0.;
                for showing in &premonitions[0..10] {
                    top_showings += showing.1; // (_commit, score, _variation)
                }
                let club_score = top_showings / 10.;
                if let Some(previous_score) = previously {
                    let orienting_factor =
                        if depth % 2 == 0 { -1. } else { 1. };
                    let lurch = orienting_factor * (club_score - previous_score);
                    tempo_lurches.push(lurch);
                }
                previously = Some(club_score);
            }
        }
        let average_tempo_lurch =
            tempo_lurches.iter().sum::<f32>()/tempo_lurches.len() as f32;
        println!("tempo lurches were {:?}, average was {}",
                 tempo_lurches, average_tempo_lurch);
        assert_eq_within_ε!(REWARD_FOR_INITIATIVE, average_tempo_lurch, 0.8);
    }

}
