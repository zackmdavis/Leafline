use std::f32::{INFINITY, NEG_INFINITY};
use std::cmp::Ordering;
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
use fnv;

use identity::{Agent, JobDescription, Team};
use life::{Commit, Patch, WorldState};
use landmark::{CENTER_OF_THE_WORLD, HIGH_COLONELCY, HIGH_SEVENTH_HEAVEN,
               LOW_COLONELCY, LOW_SEVENTH_HEAVEN, FILES};
use space::{Pinfield, Locale};
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
        for agent in &Agent::dramatis_personæ(team) {
            valuation += f32::from(world.agent_to_pinfield_ref(*agent)
                                   .pincount()) *
                figurine_valuation(*agent);
        }
        // breadth of scholarship bonus
        if world.agent_to_pinfield_ref(Agent {
                    team,
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
    valuation += 0.1 * f32::from(orange_centerism - blue_centerism);

    // a cop's favorite beat is the seventh rank
    let high_seventh = Pinfield(HIGH_SEVENTH_HEAVEN);
    let orange_beat = world.orange_cops.intersection(high_seventh).pincount();
    valuation += 0.5 * f32::from(orange_beat);
    let low_seventh = Pinfield(LOW_SEVENTH_HEAVEN);
    let blue_beat = world.blue_cops.intersection(low_seventh).pincount();
    valuation -= 0.5 * f32::from(blue_beat);

    // servants who walk behind other servants to hide must be punished
    for raw_file in &FILES {
        let file = Pinfield(*raw_file);
        let orange_servants_in_line = world.orange_servants
                                           .intersection(file)
                                           .pincount();
        // Putting a precise number on how bad extra servants on a file are
        // seems to be quite hard, and a smarter engine might choose more
        // dynamically, but half-a-point is OK, I think.
        // Wikipedia has examples where a doubled servant is worth anywhere
        // from .3 to .75 points.
        if orange_servants_in_line > 1 {
            valuation -= 0.5 * f32::from(orange_servants_in_line - 1);
        }
        let blue_servants_in_line = world.blue_servants
                                           .intersection(file)
                                           .pincount();
        if blue_servants_in_line > 1 {
            valuation += 0.5 * f32::from(blue_servants_in_line - 1);
        }
    }

    // servants should aspire to something more in life someday
    let orange_subascendants = world.orange_servants
                                    .intersection(high_seventh)
                                    .pincount();
    valuation += 1.8 * f32::from(orange_subascendants);
    let high_colonelcy = Pinfield(HIGH_COLONELCY);
    let orange_subsubascendants = world.orange_servants
                                       .intersection(high_colonelcy)
                                       .pincount();
    valuation += 0.6 * f32::from(orange_subsubascendants);
    let blue_subascendants = world.blue_servants
                                  .intersection(low_seventh)
                                  .pincount();
    valuation -= 1.8 * f32::from(blue_subascendants);
    let low_colonelcy = Pinfield(LOW_COLONELCY);
    let blue_subsubascendants = world.blue_servants
                                     .intersection(low_colonelcy)
                                     .pincount();
    valuation -= 0.6 * f32::from(blue_subsubascendants);

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

fn mvv_lva_heuristic(commit: &Commit) -> f32 {
    // https://chessprogramming.wikispaces.com/MVV-LVA
    match commit.hospitalization {
        Some(patient) => {
            (figurine_valuation(patient) - figurine_valuation(commit.patch.star))
        }
        None => 0.0,
    }
}

fn order_movements_intuitively(
        experience: &fnv::FnvHashMap<Patch, u32>,
        commits: &mut Vec<Commit>) -> Vec<Commit> {
    let mut sorted: Vec<(Commit, Option<&u32>, f32)> = Vec::with_capacity(commits.len());
    for c in commits {
        sorted.push((*c, experience.get(&c.patch), mvv_lva_heuristic(&c)));
    }
    sorted.sort_unstable_by(|a, b| {
        match b.1.cmp(&a.1) {
            Ordering::Equal => b.2.partial_cmp(&a.2).unwrap_or(Ordering::Equal),
            other => other,
        }
    });
    sorted.iter().map(|c| { c.0 }).collect()
}

pub type Variation = Vec<Patch>;


#[allow(ptr_arg)]
pub fn pagan_variation_format(variation: &Variation) -> String {
    variation.iter()
             .map(|p| p.abbreviated_pagan_movement_rune())
             .collect::<Vec<_>>()
             .join(" ")
}

pub trait Memory: Clone + Send {
    fn recombine(&mut self, other: Self);
    fn flash(patch: Patch) -> Self;
    fn blank() -> Self;
    fn readable(&self) -> String;
}

impl Memory for Patch {
    fn recombine(&mut self, other: Self) {
        self.star = other.star;
        self.whence = other.whence;
        self.whither = other.whither;
    }

    fn flash(patch: Patch) -> Self {
        patch
    }
    fn blank() -> Self {
        // deliberately illegal hyperspace warp from the Figurehead; possibly useful for debugging.
        // a "blank" commit isn't really a thing.
        Patch {
            star: Agent::new(Team::Orange, JobDescription::Figurehead),
            whence: Locale::new(0, 0),
            whither: Locale::new(7, 7),
        }
    }

    fn readable(&self) -> String {
        self.abbreviated_pagan_movement_rune()
    }

}

impl Memory for Variation {
    fn recombine(&mut self, other: Self) {
        self.extend(other);
    }

    fn flash(patch: Patch) -> Self {
        vec![patch]
    }
    fn blank() -> Self {
        vec![]
    }

    fn readable(&self) -> String {
        pagan_variation_format(&self)
    }
}

#[derive(Clone)]
pub struct Lodestar<T: Memory> {
    pub score: f32,
    pub memory: T,
}

impl<T: Memory> Lodestar<T> {
    fn new(score: f32, memory: T) -> Self {
        Self {
            score,
            memory,
        }
    }
}

impl<T: Memory> fmt::Debug for Lodestar<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f,
               "Lodestar {{ score: {}, memory: {} }}",
               self.score,
               self.memory.readable())
    }
}

#[derive(Eq,PartialEq,Hash)]
pub struct SpaceTime {
    world_state: WorldState,
    instant: i8,
}


impl SpaceTime {
    fn new(world_state: WorldState, instant: i8) -> Self {
        Self { world_state, instant }
    }
}


#[allow(too_many_arguments)]
pub fn α_β_negamax_search<T: Memory>(
    world: WorldState, depth: i8, mut α: f32, β: f32,
    memory_bank: Arc<parking_lot::Mutex<LruCache<SpaceTime, Lodestar<T>,
                                    BuildHasherDefault<XxHash>>>>,
    intuition_bank: Arc<parking_lot::Mutex<fnv::FnvHashMap<Patch, u32>>>,
    quiet: Option<u8>)
        -> Lodestar<T> {

    let mut premonitions = world.reckless_lookahead();
    let mut optimum = NEG_INFINITY;
    let mut optimand = T::blank();
    if depth <= 0 || premonitions.is_empty() {
        let potential_score = orientation(world.initiative) * score(world);
        match quiet {
            None => {
                return Lodestar::new(potential_score, T::blank());
            },
            Some(extension) => {
                if depth.abs() >= extension as i8 {
                    return Lodestar::new(potential_score, T::blank());
                }
                premonitions = premonitions.into_iter()
                    .filter(|c| c.hospitalization.is_some())
                    .collect::<Vec<_>>();
                if premonitions.is_empty() {
                    return Lodestar::new(potential_score, T::blank())
                } else {
                    optimum = potential_score;
                }
            }
        }
    };

    // Note: if sorting by heuristic were sufficiently expensive, it would, on balance, be better
    // to do so only at the higher levels of the tree. From some minor empiric testing, though,
    // sorting only at depth >= 1 has no performance impact, and at depth >=2 has a negative
    // performance impact. So that's not the way to go.
    {
        let experience = intuition_bank.lock();
        premonitions = order_movements_intuitively(&experience, &mut premonitions)
    }
    for premonition in premonitions {
        let mut value = NEG_INFINITY;  // can't hurt to be pessimistic
        let mut memory: T = T::flash(premonition.patch);
        let cached: bool;
        let space_time = SpaceTime::new(premonition.tree, depth);
        {
            let mut open_vault = memory_bank.lock();
            let remembered_lodestar_maybe = open_vault.get_mut(&space_time);
            match remembered_lodestar_maybe {
                Some(remembered_lodestar) => {
                    cached = true;
                    value = remembered_lodestar.score;
                    memory.recombine(remembered_lodestar.memory.clone());
                }
                None => { cached = false; }
            };
        }

        if !cached {
            let mut lodestar = α_β_negamax_search(
                premonition.tree, depth - 1,
                -β, -α,
                memory_bank.clone(), intuition_bank.clone(),
                quiet
            );
            lodestar.score *= -1.;  // nega-
            value = lodestar.score;
            memory.recombine(lodestar.memory.clone());
            memory_bank.lock().insert(
                space_time,
                lodestar,
            );
        }

        if value > optimum {
            optimum = value;
            optimand = memory;
        }
        if value > α {
            α = value;
        }
        if α >= β {
            if depth > 0 { // not a quietness extension
                let mut open_vault = intuition_bank.lock();
                let intuition = open_vault.entry(premonition.patch)
                    .or_insert(0);
                *intuition += 2u32.pow(depth as u32);
            }
            break;  // cutoff!
        }
    }
    Lodestar::new(optimum, optimand)
}


pub fn déjà_vu_table_size_bound<T: Memory>(gib: f32) -> usize {

    let bound = usize::from(Bytes::gibi(gib)) /
        (mem::size_of::<SpaceTime>() + mem::size_of::<Lodestar<T>>());
    bound
}


pub fn potentially_timebound_kickoff<T: 'static + Memory>(
    world: &WorldState, depth: u8,
    extension_maybe: Option<u8>,
    nihilistically: bool,
    deadline_maybe: Option<time::Timespec>,
    intuition_bank: Arc<parking_lot::Mutex<fnv::FnvHashMap<Patch, u32>>>,
    déjà_vu_bound: f32)
        -> Option<Vec<(Commit, f32, T)>> {
    let déjà_vu_table: LruCache<SpaceTime, Lodestar<T>,
                                BuildHasherDefault<XxHash>> =
        LruCache::with_hash_state(déjà_vu_table_size_bound::<T>(déjà_vu_bound),
                                  Default::default());
    let memory_bank = Arc::new(parking_lot::Mutex::new(déjà_vu_table));
    let mut premonitions = if nihilistically {
        world.reckless_lookahead()
    } else {
        world.lookahead()
    };
    {
        let experience = intuition_bank.lock();
        premonitions = order_movements_intuitively(&experience, &mut premonitions)
    }
    let mut forecasts = Vec::with_capacity(40);
    let mut time_radios: Vec<(Commit, mpsc::Receiver<Lodestar<T>>)> = Vec::new();
    for &premonition in &premonitions {
        let travel_memory_bank = memory_bank.clone();
        let travel_intuition_bank = intuition_bank.clone();
        let (tx, rx) = mpsc::channel();
        let explorer_radio = tx.clone();
        time_radios.push((premonition, rx));
        thread::spawn(move || {
            let search_hit: Lodestar<T> = α_β_negamax_search(
                premonition.tree, (depth - 1) as i8,
                NEG_INFINITY, INFINITY,
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
            if let Ok(search_hit) = time_radios[i].1.try_recv() {
                let value = -search_hit.score;
                let mut full_variation = T::flash(premonition.patch);
                full_variation.recombine(search_hit.memory);
                forecasts.push((premonition, value, full_variation));
                time_radios.swap_remove(i);
            }
        }
        thread::sleep(Duration::from_millis(2));
        debug!("waiting for {} of {} first-movement search threads",
               time_radios.len(), premonitions.len())
    }
    forecasts.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
    Some(forecasts)
}


pub fn kickoff<T: 'static + Memory>(world: &WorldState, depth: u8, extension: Option<u8>,
               nihilistically: bool, déjà_vu_bound: f32)
                   -> Vec<(Commit, f32, T)> {
    let experience_table: fnv::FnvHashMap<Patch, u32> = fnv::FnvHashMap::default();
    let intuition_bank = Arc::new(parking_lot::Mutex::new(experience_table));
    potentially_timebound_kickoff::<T>(world, depth, extension, nihilistically, None,
                                  intuition_bank, déjà_vu_bound).unwrap()
}


pub fn iterative_deepening_kickoff<T: 'static + Memory>(world: &WorldState, timeout: time::Duration,
                                   nihilistically: bool, déjà_vu_bound: f32)
                                   -> (Vec<(Commit, f32, T)>, u8) {
    let deadline = time::get_time() + timeout;
    let mut depth = 1;
    let experience_table = fnv::FnvHashMap::default();
    let intuition_bank = Arc::new(parking_lot::Mutex::new(experience_table));
    let mut forecasts = potentially_timebound_kickoff(
        world, depth, None, nihilistically, None,
        intuition_bank.clone(),
        déjà_vu_bound).unwrap();
    while let Some(prophecy) = potentially_timebound_kickoff::<T>(
            world, depth, None, nihilistically, Some(deadline),
            intuition_bank.clone(), déjà_vu_bound) {
        forecasts = prophecy;
        depth += 1;
    }
    (forecasts, depth-1)
}


#[allow(needless_pass_by_value)] // `depth_sequence`
pub fn fixed_depth_sequence_kickoff<T: 'static + Memory>(world: &WorldState, depth_sequence: Vec<u8>,
                                    nihilistically: bool, déjà_vu_bound: f32)
                                    -> Vec<(Commit, f32, T)> {
    let mut depths = depth_sequence.iter();
    let experience_table = fnv::FnvHashMap::default();
    let intuition_bank = Arc::new(parking_lot::Mutex::new(experience_table));
    let mut forecasts = potentially_timebound_kickoff::<T>(
        world, *depths.next().expect("`depth_sequence` should be nonempty"),
        None, nihilistically, None, intuition_bank.clone(),
        déjà_vu_bound
    ).unwrap();
    for &depth in depths {
        forecasts = potentially_timebound_kickoff::<T>(
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
    use super::{REWARD_FOR_INITIATIVE, kickoff, score, SpaceTime, Variation};
    use space::Locale;
    use life::{WorldState, Patch};
    use fnv;
    use twox_hash::XxHash;
    use std::hash::Hash;
    use std::collections::hash_map;
    use identity::{Agent, JobDescription, Team};

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
    fn benchmark_hashing_spacetime_fnv(b: &mut Bencher) {
        let w = WorldState::new();
        let st = SpaceTime::new(w, 3);
        let mut hasher = fnv::FnvHasher::default();

        b.iter(|| {
            for _ in 0..1000 {
                st.hash(&mut hasher);
            }
        });
    }

    #[bench]
    fn benchmark_hashing_spacetime_xx(b: &mut Bencher) {
        let w = WorldState::new();
        let mut hasher = XxHash::default();
        let st = SpaceTime::new(w, 3);

        b.iter(|| {
            for _ in 0..1000 {
                st.hash(&mut hasher);
            }
        });
    }

    #[bench]
    fn benchmark_hashing_spacetime_sip(b: &mut Bencher) {
        let w = WorldState::new();
        let mut hasher = hash_map::DefaultHasher::new();
        let st = SpaceTime::new(w, 3);

        b.iter(|| {
            for _ in 0..1000 {
                st.hash(&mut hasher);
            }
        });
    }

    #[bench]
    fn benchmark_hashing_patch_fnv(b: &mut Bencher) {
        let mut hasher = fnv::FnvHasher::default();
        let p = Patch {
            star: Agent {
                team: Team::Orange,
                job_description: JobDescription::Figurehead,
            },
            whence: Locale::new(1, 2),
            whither: Locale::new(3, 4)
        };

        b.iter(|| {
            for _ in 0..1000 {
                p.hash(&mut hasher);
            }
        });
    }

    #[bench]
    fn benchmark_hashing_patch_xx(b: &mut Bencher) {
        let mut hasher = XxHash::default();
        let p = Patch {
            star: Agent {
                team: Team::Orange,
                job_description: JobDescription::Figurehead,
            },
            whence: Locale::new(1, 2),
            whither: Locale::new(3, 4)
        };

        b.iter(|| {
            for _ in 0..1000 {
                p.hash(&mut hasher);
            }
        });
    }

    #[bench]
    fn benchmark_hashing_patch_sip(b: &mut Bencher) {
        let mut hasher = hash_map::DefaultHasher::new();
        let p = Patch {
            star: Agent {
                team: Team::Orange,
                job_description: JobDescription::Figurehead,
            },
            whence: Locale::new(1, 2),
            whither: Locale::new(3, 4)
        };

        b.iter(|| {
            for _ in 0..1000 {
                p.hash(&mut hasher);
            }
        });
    }

    #[bench]
    fn benchmark_scoring(b: &mut Bencher) {
        b.iter(|| score(WorldState::new()));
    }

    #[bench]
    fn benchmark_kickoff_depth_1(b: &mut Bencher) {
        let ws = WorldState::new();
        b.iter(|| kickoff::<Patch>(&ws, 1, None, true, MOCK_DÉJÀ_VU_BOUND));
    }

    #[bench]
    fn benchmark_kickoff_depth_2_arbys(b: &mut Bencher) {
        let ws = WorldState::new();
        b.iter(|| kickoff::<Patch>(&ws, 2, None, true, MOCK_DÉJÀ_VU_BOUND));
    }

    #[bench]
    fn benchmark_kickoff_depth_2_carefully(b: &mut Bencher) {
        let ws = WorldState::new();
        b.iter(|| kickoff::<Patch>(&ws, 2, None, false, MOCK_DÉJÀ_VU_BOUND));
    }

    #[bench]
    fn benchmark_kickoff_depth_3(b: &mut Bencher) {
        let ws = WorldState::new();
        b.iter(|| kickoff::<Patch>(&ws, 3, None, true, MOCK_DÉJÀ_VU_BOUND));
    }

    #[test]
    #[ignore]  // more research is needed
    fn concerning_short_circuiting_upon_finding_critical_endangerment() {
        let ws = WorldState::reconstruct("7K/r7/1r6/8/8/8/8/7k b -");
        let start = time::get_time();
        kickoff::<Variation>(&ws, 30, None, true, MOCK_DÉJÀ_VU_BOUND);
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
        let ws = WorldState::reconstruct("8/q1P1k/8/8/8/8/6PP/7K w - -");
        // looking ahead 3 movements allows the Leafline AI to catch the
        // split, whereby transforming into a pony (rather than
        // transitioning into a princess, as would usually be
        // expected) endangers both the blue princess and figurehead
        let tops = kickoff::<Variation>(&ws, 3, None, true, MOCK_DÉJÀ_VU_BOUND);
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
        let advisory = kickoff::<Variation>(&world, depth, None, true, MOCK_DÉJÀ_VU_BOUND);

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

        let negadvisory = kickoff::<Variation>(&negaworld, depth, None, true, MOCK_DÉJÀ_VU_BOUND);

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
            let world = WorldState::reconstruct(world_runeset);
            let mut previously = None;
            for &depth in &[2, 3, 4] {
                let premonitions = kickoff::<Variation>(&world, depth, None, true, 1.0);
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

    #[test]
    fn concerning_lazy_servants() {
        let orange_doubled = WorldState::reconstruct("k7/pp6/8/8/8/P7/P7/K7 w - -");
        let orange_not_doubled = WorldState::reconstruct("k7/pp6/8/8/8/8/PP6/K7 w - -");
        assert!(score(orange_doubled) < score(orange_not_doubled));
    }
}
