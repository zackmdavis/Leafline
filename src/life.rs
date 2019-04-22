//! the `life` module of the Leafline oppositional strategy game engine
use std::default::Default;
use std::fmt;

use space::{Locale, Pinfield};
use identity::{Agent, JobDescription, Team};
use motion::{FIGUREHEAD_MOVEMENT_TABLE, PONY_MOVEMENT_TABLE};
use ansi_term::Colour as Color;


/// represents the movement of a figurine
#[derive(Eq,PartialEq,Debug,Copy,Clone,Hash,RustcEncodable,RustcDecodable)]
pub struct Patch {
    pub star: Agent,
    pub whence: Locale,
    pub whither: Locale,
}

impl Patch {
    pub fn concerns_secret_service(&self) -> bool {
        self.star.job_description == JobDescription::Figurehead &&
        (self.whence.file as i8 - self.whither.file as i8).abs() == 2
    }

    pub fn concerns_servant_ascension(&self) -> bool {
        let admirality = match self.star.team {
            Team::Orange => 7,
            Team::Blue => 0,
        };
        self.star.job_description == JobDescription::Servant &&
            self.whither.rank == admirality
    }

    pub fn abbreviated_pagan_movement_rune(&self) -> String {
        if self.concerns_secret_service() {
            match self.whither.file {
                6 => "O–O".to_owned(),
                2 => "O–O–O".to_owned(),
                _ => {
                    moral_panic!("secret service movement didn't end with \
                                  figurehead on file 2 or 6")
                }
            }
        } else {
            format!("{}{}",
                    self.star.to_pagan_movement_rune_prefix(),
                    self.whither.to_algebraic())
        }
    }
}


/// represents the outcome of a team's turn with a `patch` governing
/// the figurine moved, the state of the world after the turn (`tree`),
/// and whether an opposing figurine was stunned and put in the hospital,
/// and if so, which one
#[derive(Eq,PartialEq,Debug,Copy,Clone,Hash)]
pub struct Commit {
    pub patch: Patch,
    pub tree: WorldState,
    pub hospitalization: Option<Agent>,
    pub ascension: Option<Agent>,
}

impl Commit {
    pub fn pagan_movement_rune(&self) -> String {
        format!("{movement}{endangerment}{ascension}",
                movement = if self.hospitalization.is_some() {
                    format!("{}x{}",
                            self.patch.star.to_pagan_movement_rune_prefix(),
                            self.patch.whither.to_algebraic())
                } else {
                    self.patch.abbreviated_pagan_movement_rune()
                },
                endangerment = if self.tree
                                      .in_critical_endangerment(self.tree
                                                                    .initiative) {
                    "+"
                } else {
                    ""
                },
                ascension = match self.ascension {
                    Some(ascended_form) => {
                        format!("={}", ascended_form.to_pagan_movement_rune_prefix())
                    }
                    None => "".to_owned(),
                })
    }
}

impl fmt::Display for Commit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let hospital_report = match self.hospitalization {
            Some(stunning_victim) => format!(", stunning {}", stunning_victim),
            None => "".to_owned(),
        };
        let ascension_report = match self.ascension {
            Some(ascended_form) => {
                match ascended_form.job_description {
                    JobDescription::Servant => {
                        moral_panic!("servant purportedly 'ascending'to his own \
                                      station (?!)")
                    }
                    JobDescription::Pony => {
                        format!(", transforming into {}", ascended_form)
                    }
                    JobDescription::Cop => {
                        format!(", being brevetted to {}", ascended_form)
                    }
                    JobDescription::Scholar | JobDescription::Princess => {
                        format!(", transitioning into {}", ascended_form)
                    }
                    JobDescription::Figurehead => {
                        moral_panic!("servant purportedly ascending to figurehead \
                                      (?!)")
                    }
                }
            }
            None => "".to_owned(),
        };
        let report = hospital_report + &ascension_report;
        write!(f,
               "{}{} ({} from {} to {}{})",
               match self.patch.star.team {
                   Team::Orange => "",
                   Team::Blue => "..",
               },
               self.pagan_movement_rune(),
               self.patch.star,
               self.patch.whence.to_algebraic(),
               self.patch.whither.to_algebraic(),
               report)
    }
}


#[derive(Eq,PartialEq,Debug,Copy,Clone,Hash)]
pub struct WorldState {
    pub initiative: Team,

    pub orange_servants: Pinfield,
    pub orange_ponies: Pinfield,
    pub orange_scholars: Pinfield,
    pub orange_cops: Pinfield,
    pub orange_princesses: Pinfield,
    pub orange_figurehead: Pinfield,

    pub blue_servants: Pinfield,
    pub blue_ponies: Pinfield,
    pub blue_scholars: Pinfield,
    pub blue_cops: Pinfield,
    pub blue_princesses: Pinfield,
    pub blue_figurehead: Pinfield,
    pub service_eligibility: u8,
    pub passing_by_locale: Option<Locale>,
}

const ORANGE_FIGUREHEAD_START: Locale = Locale { rank: 0, file: 4 };
const BLUE_FIGUREHEAD_START: Locale = Locale { rank: 7, file: 4 };
const ORANGE_WEST_ELIGIBILITY: u8 = 0b1;
const ORANGE_EAST_ELIGIBILITY: u8 = 0b10;
const BLUE_WEST_ELIGIBILITY: u8 = 0b100;
const BLUE_EAST_ELIGIBILITY: u8 = 0b1000;


impl Default for WorldState {
    fn default() -> Self {
        let mut orange_servant_locales = Vec::new();
        let mut blue_servant_locales = Vec::new();
        for f in 0..8 {
            orange_servant_locales.push(Locale::new(1, f));
            blue_servant_locales.push(Locale::new(6, f));
        }
        WorldState {
            initiative: Team::Orange,

            orange_servants: Pinfield::init(&orange_servant_locales),
            orange_ponies: Pinfield::init(&[Locale::new(0, 1),
                                            Locale::new(0, 6)]),
            orange_scholars: Pinfield::init(&[Locale::new(0, 2),
                                              Locale::new(0, 5)]),
            orange_cops: Pinfield::init(&[Locale::new(0, 0), Locale::new(0, 7)]),
            orange_princesses: Pinfield::init(&[Locale::new(0, 3)]),
            orange_figurehead: Pinfield::init(&[ORANGE_FIGUREHEAD_START]),

            blue_servants: Pinfield::init(&blue_servant_locales),
            blue_ponies: Pinfield::init(&[Locale::new(7, 1), Locale::new(7, 6)]),
            blue_scholars: Pinfield::init(&[Locale::new(7, 2),
                                            Locale::new(7, 5)]),
            blue_cops: Pinfield::init(&[Locale::new(7, 0), Locale::new(7, 7)]),
            blue_princesses: Pinfield::init(&[Locale::new(7, 3)]),
            blue_figurehead: Pinfield::init(&[BLUE_FIGUREHEAD_START]),
            service_eligibility: 0b1111,
            passing_by_locale: None,
        }
    }
}

impl WorldState {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_except_empty() -> Self {
        Self {
            initiative: Team::Orange,

            orange_servants: Pinfield::new(),
            orange_ponies: Pinfield::new(),
            orange_scholars: Pinfield::new(),
            orange_cops: Pinfield::new(),
            orange_princesses: Pinfield::new(),
            orange_figurehead: Pinfield::new(),
            blue_servants: Pinfield::new(),
            blue_ponies: Pinfield::new(),
            blue_scholars: Pinfield::new(),
            blue_cops: Pinfield::new(),
            blue_princesses: Pinfield::new(),
            blue_figurehead: Pinfield::new(),
            service_eligibility: 0,
            passing_by_locale: None,
        }
    }

    pub fn agent_to_pinfield_ref(&self, agent: Agent) -> &Pinfield {
        match_agent!(
            agent,
            Orange, Servant => &self.orange_servants,
            Orange, Pony => &self.orange_ponies,
            Orange, Scholar => &self.orange_scholars,
            Orange, Cop => &self.orange_cops,
            Orange, Princess => &self.orange_princesses,
            Orange, Figurehead => &self.orange_figurehead,
            Blue, Servant => &self.blue_servants,
            Blue, Pony => &self.blue_ponies,
            Blue, Scholar => &self.blue_scholars,
            Blue, Cop => &self.blue_cops,
            Blue, Princess => &self.blue_princesses,
            Blue, Figurehead => &self.blue_figurehead
        )
    }

    // XXX Less code duplication, but still not ideal. the macro
    // system is really confusing to me: it looks like you can have a
    // macro that generates code that compiles, but that the code with
    // the macro does not compile. O.o
    pub fn agent_to_pinfield_mutref(&mut self, agent: Agent) -> &mut Pinfield {
        match_agent!(
            agent,
            Orange, Servant => &mut self.orange_servants,
            Orange, Pony => &mut self.orange_ponies,
            Orange, Scholar => &mut self.orange_scholars,
            Orange, Cop => &mut self.orange_cops,
            Orange, Princess => &mut self.orange_princesses,
            Orange, Figurehead => &mut self.orange_figurehead,
            Blue, Servant => &mut self.blue_servants,
            Blue, Pony => &mut self.blue_ponies,
            Blue, Scholar => &mut self.blue_scholars,
            Blue, Cop => &mut self.blue_cops,
            Blue, Princess => &mut self.blue_princesses,
            Blue, Figurehead => &mut self.blue_figurehead
        )
    }

    pub fn is_being_leered_at_by(&self, locale: Locale, team: Team) -> bool {
        let agent = Agent::new(team.opposition(), JobDescription::Figurehead);
        let pinfield = self.agent_to_pinfield_ref(agent);
        let mut tree = self.except_replaced_subboard(agent, pinfield.alight(locale));
        tree.initiative = team;
        let prems = tree.lookahead_without_secret_service(true);
        prems.iter().any(|c| (*c).patch.whither == locale)
    }

    pub fn preserve(&self) -> String {
        fn void_void_run_length(work: &mut String, counter: &mut u8) {
            work.push(counter.to_string().chars().next().unwrap());
            *counter = 0;
        }

        let mut book = String::with_capacity(
            // pessimistic board storage + rank delimiters +
            // metadata
            // ≟ 64 + 7 + 14 =
            85);

        let mut void_run_length = 0;
        for rank in (0..8).rev() {
            for file in 0..8 {
                let agent_maybe = self.occupying_agent(
                    Locale::new(rank, file));
                match agent_maybe {
                    Some(agent) => {
                        if void_run_length > 0 {
                            void_void_run_length(
                                &mut book, &mut void_run_length);
                        }
                        book.push(agent.to_preservation_rune());
                    }
                    None => {
                        void_run_length += 1;
                    }
                }
            }
            if void_run_length > 0 {
                void_void_run_length(&mut book, &mut void_run_length);
            }
            if rank > 0 {
                book.push('/')
            }
        }
        let initiative_indication_rune = match self.initiative {
            // TODO: think of some remotely plausible rationalization for 'w'
            Team::Orange => 'w',
            Team::Blue => 'b',
        };
        book.push(' ');
        book.push(initiative_indication_rune);
        book.push(' ');
        let mut any_service = false;
        for &(service_eligibility, eligibility_rune) in
            &[(self.orange_east_service_eligibility(), 'K'),
              (self.orange_west_service_eligibility(), 'Q'),
              (self.blue_east_service_eligibility(), 'k'),
              (self.blue_west_service_eligibility(), 'q')] {
            if service_eligibility {
                book.push(eligibility_rune);
                if !any_service {
                    any_service = true;
                }
            }
        }
        if !any_service {
            book.push('-');
        }
        book.push(' ');
        match self.passing_by_locale {
            Some(locale) =>  { book.push_str(&locale.to_algebraic()) },
            None => { book.push('-'); }
        }
        book
    }

    pub fn orange_east_service_eligibility(&self) -> bool {
        self.service_eligibility & ORANGE_EAST_ELIGIBILITY > 0
    }

    pub fn orange_west_service_eligibility(&self) -> bool {
        self.service_eligibility & ORANGE_WEST_ELIGIBILITY > 0
    }

    pub fn blue_east_service_eligibility(&self) -> bool {
        self.service_eligibility & BLUE_EAST_ELIGIBILITY > 0
    }

    pub fn blue_west_service_eligibility(&self) -> bool {
        self.service_eligibility & BLUE_WEST_ELIGIBILITY > 0
    }


    fn service_eligibility_bit(&self, team: Team, west: bool) -> u8 {
        match (team, west) {
            (Team::Orange, true) => ORANGE_WEST_ELIGIBILITY,
            (Team::Orange, false) => ORANGE_EAST_ELIGIBILITY,
            (Team::Blue, true) => BLUE_WEST_ELIGIBILITY,
            (Team::Blue, false) => BLUE_EAST_ELIGIBILITY,
        }
    }

    fn clear_service_eligibility(&mut self, team: Team, west: bool) {
        self.service_eligibility &= !(self.service_eligibility_bit(team, west));
    }

    fn set_service_eligibility(&mut self, team: Team, west: bool) {
        self.service_eligibility |= self.service_eligibility_bit(team, west);
    }

    pub fn clear_orange_east_service_eligibility(&mut self) {
        self.clear_service_eligibility(Team::Orange, false);
    }

    pub fn clear_orange_west_service_eligibility(&mut self) {
        self.clear_service_eligibility(Team::Orange, true);
    }

    pub fn clear_blue_east_service_eligibility(&mut self) {
        self.clear_service_eligibility(Team::Blue, false);
    }

    pub fn clear_blue_west_service_eligibility(&mut self) {
        self.clear_service_eligibility(Team::Blue, true);
    }

    pub fn set_orange_east_service_eligibility(&mut self) {
        self.set_service_eligibility(Team::Orange, false);
    }

    pub fn set_orange_west_service_eligibility(&mut self) {
        self.set_service_eligibility(Team::Orange, true);
    }

    pub fn set_blue_east_service_eligibility(&mut self) {
        self.set_service_eligibility(Team::Blue, false);
    }

    pub fn set_blue_west_service_eligibility(&mut self) {
        self.set_service_eligibility(Team::Blue, true);
    }

    pub fn reconstruct(scan: &str) -> Self {
        let mut rank = 7;
        let mut file = 0;
        let mut world = WorldState::new_except_empty();
        let replaced = scan.replace("X", " ");
        let mut volumes = replaced.split(' ');
        let positional_scan = volumes.next();
        for rune in positional_scan.unwrap().chars() {
            match rune {
                '/' => {
                    file = 0;
                    rank -= 1;
                }
                empty_locales @ '0' ... '8' => {
                    let file_offset: u8 = empty_locales.to_string()
                                                       .parse()
                                                       .unwrap();
                    file += file_offset;
                }
                r => {
                    let agent = Agent::from(r);
                    let derived_pinfield;
                    {
                        let hot_pinfield = world.agent_to_pinfield_ref(agent);
                        derived_pinfield = hot_pinfield.alight(
                            Locale::new(rank, file));
                    }
                    // XXX: this isn't Clojure; copying a
                    // datastructure with a small diff actually has costs
                    world = world.except_replaced_subboard(
                        agent, derived_pinfield);
                    file += 1;
                }
            }
        }
        let rune_of_those_with_initiative = volumes.next()
                                                   .unwrap()
                                                   .chars()
                                                   .next()
                                                   .unwrap();
        world.initiative = match rune_of_those_with_initiative {
            'w' => Team::Orange,
            'b' => Team::Blue,
            _ => {
                moral_panic!("Non-initiative-preserving-rune passed to a match \
                              expecting such")
            }
        };
        let secret_service_eligibilities = volumes.next().unwrap();
        for eligibility in secret_service_eligibilities.chars() {
            match eligibility {
                'K' => {
                    world.set_orange_east_service_eligibility();
                }
                'Q' => {
                    world.set_orange_west_service_eligibility();
                }
                'k' => {
                    world.set_blue_east_service_eligibility();
                }
                'q' => {
                    world.set_blue_west_service_eligibility();
                }
                '-' => {
                    break;
                }
                r => {
                    moral_panic!(format!("non-eligibility rune '{}'", r));
                }
            }
        }

        let passing_by_locale = volumes.next().expect("expected passing-by locale rune");
        if passing_by_locale == "-" {
            world.passing_by_locale = None;
        } else {
            world.passing_by_locale = Some(Locale::from_algebraic(passing_by_locale));
        }
        world
    }

    pub fn except_replaced_subboard(&self, for_whom: Agent, subboard: Pinfield)
                                    -> Self {
        let mut resultant_state = *self;
        resultant_state.agent_to_pinfield_mutref(for_whom).0 = subboard.0;
        resultant_state
    }

    pub fn occupied_by(&self, team: Team) -> Pinfield {
        match team {
            Team::Orange => {
                self.orange_servants
                    .union(self.orange_ponies)
                    .union(self.orange_scholars)
                    .union(self.orange_cops)
                    .union(self.orange_princesses)
                    .union(self.orange_figurehead)
            }
            Team::Blue => {
                self.blue_servants
                    .union(self.blue_ponies)
                    .union(self.blue_scholars)
                    .union(self.blue_cops)
                    .union(self.blue_princesses)
                    .union(self.blue_figurehead)
            }
        }
    }

    pub fn occupied(&self) -> Pinfield {
        self.occupied_by(Team::Orange).union(self.occupied_by(Team::Blue))
    }

    pub fn unoccupied(&self) -> Pinfield {
        self.occupied().invert()
    }

    pub fn occupying_affiliated_agent(&self, at: Locale, team: Team) -> Option<Agent> {
        for agent in Agent::dramatis_personæ(team) {
            if self.agent_to_pinfield_ref(agent).query(at) {
                return Some(agent);
            }
        }
        None
    }

    fn occupying_agent(&self, at: Locale) -> Option<Agent> {
        for &team in &Team::league() {
            let agent_maybe = self.occupying_affiliated_agent(at, team);
            if agent_maybe.is_some() {
                return agent_maybe;
            }
        }
        None
    }

    pub fn apply(&self, patch: Patch) -> Commit {
        // subboard of moving figurine
        let backstory = self.agent_to_pinfield_ref(patch.star);
        // subboard of moving figurine after move
        let derived_subboard = backstory.transit(patch.whence, patch.whither);
        // insert subboard into post-patch world-model
        let mut tree = self.except_replaced_subboard(patch.star, derived_subboard);
        match patch.star.job_description {
            JobDescription::Figurehead => {
                match patch.star.team {
                    Team::Orange => {
                        tree.clear_orange_east_service_eligibility();
                        tree.clear_orange_west_service_eligibility();
                    }
                    Team::Blue => {
                        tree.clear_blue_east_service_eligibility();
                        tree.clear_blue_west_service_eligibility();
                    }
                }
            }
            JobDescription::Cop => {
                match (patch.whence.file, patch.star.team) {
                    (0, Team::Orange) => {
                        tree.clear_orange_west_service_eligibility();
                    }
                    (7, Team::Orange) => {
                        tree.clear_orange_east_service_eligibility();
                    }
                    (0, Team::Blue) => {
                        tree.clear_blue_west_service_eligibility();
                    }
                    (7, Team::Blue) => {
                        tree.clear_blue_east_service_eligibility();
                    }
                    _ => {}
                }
            }
            _ => {}
        }


        if patch.concerns_secret_service() {
            let cop_agent = Agent::new(patch.star.team, JobDescription::Cop);
            let (start_file, end_file) = match patch.whither.file {
                6 => (7, 5),
                2 => (0, 3),
                _ => {
                    moral_panic!("This looked like a Secret Service commit, but it \
                                  is not")
                }
            };

            let secret_derived_subboard = tree.agent_to_pinfield_ref(cop_agent)
                                              .transit(Locale::new(patch.whither
                                                                        .rank,
                                                                   start_file),
                                                       Locale::new(patch.whither
                                                                        .rank,
                                                                   end_file));
            tree = tree.except_replaced_subboard(cop_agent, secret_derived_subboard);
        }

        // was anyone stunned?
        let opposition = tree.initiative.opposition();
        let mut hospitalization = self.occupying_affiliated_agent(patch.whither,
                                                                  opposition);
        let mut ambulance_target = patch.whither;

        if hospitalization.is_none() &&
            patch.star.job_description == JobDescription::Servant {
            if let Some(passed_by) = self.passing_by_locale {
                if passed_by == patch.whither {

                    let direction = match opposition {
                        Team::Orange => (1, 0),
                        Team::Blue => (-1, 0)
                    };
                    ambulance_target = passed_by.displace(direction).unwrap();
                    hospitalization = self.occupying_affiliated_agent(ambulance_target,
                                                                      opposition);
                }
            }
        }
        if let Some(stunned) = hospitalization {
            // if someone was stunned, put her or him in the hospital
            let further_derived_subboard = tree.agent_to_pinfield_ref(stunned)
                                               .quench(ambulance_target);
            tree = tree.except_replaced_subboard(stunned, further_derived_subboard);
        }

        tree.initiative = opposition;
        if patch.star.job_description == JobDescription::Servant &&
           (patch.whither.rank as i8 - patch.whence.rank as i8).abs() == 2 {
                let direction = match patch.star.team {
                    Team::Orange => (1, 0),
                    Team::Blue => (-1, 0)
                };
                tree.passing_by_locale = patch.whence.displace(direction);
        } else {
            tree.passing_by_locale = None;
        }
        Commit {
            patch,
            tree,
            hospitalization,
            ascension: None,
        }
    }

    pub fn in_critical_endangerment(&self, team: Team) -> bool {
        let mut contingency = *self;
        contingency.initiative = team.opposition();
        let premonitions = contingency.reckless_lookahead();
        for premonition in &premonitions {
            if let Some(patient) = premonition.hospitalization {
                if patient.job_description == JobDescription::Figurehead {
                    return true;
                }
            }
        }
        false
    }

    pub fn careful_apply(&self, patch: Patch) -> Option<Commit> {
        let force_commit = self.apply(patch);
        if force_commit.tree.in_critical_endangerment(self.initiative) {
            None
        } else {
            Some(force_commit)
        }
    }

    fn subpredict(&self, premonitions: &mut Vec<Commit>, premonition: Commit) {
        if premonition.patch.concerns_servant_ascension() {
            for ascended in Agent::dramatis_personæ(premonition.patch.star.team) {
                if ascended.job_description == JobDescription::Servant ||
                   ascended.job_description == JobDescription::Figurehead {
                    continue;
                }
                let mut ascendency = premonition;
                ascendency.ascension = Some(ascended);
                let vessel_pinfield =
                    premonition.tree
                               .agent_to_pinfield_ref(premonition.patch.star)
                               .quench(premonition.patch.whither);
                let ascended_pinfield = premonition.tree
                                                   .agent_to_pinfield_ref(ascended)
                                                   .alight(premonition.patch
                                                                      .whither);
                ascendency.tree =
                    ascendency.tree
                              .except_replaced_subboard(premonition.patch.star,
                                                        vessel_pinfield)
                              .except_replaced_subboard(ascended, ascended_pinfield);
                premonitions.push(ascendency);
            }
        } else {
            premonitions.push(premonition);
        }
    }

    pub fn predict(&self, premonitions: &mut Vec<Commit>, patch: Patch,
                   nihilistically: bool) {
        if nihilistically {  // enjoy Arby's
            let premonition = self.apply(patch);
            self.subpredict(premonitions, premonition);
        } else {
            let premonition_maybe = self.careful_apply(patch);
            if let Some(premonition) = premonition_maybe {
                self.subpredict(premonitions, premonition);
            }
        }
    }

    /// "... what he knows throws the blows when he goes to the
    /// fight. And he'll win the whole thing 'fore he enters the ring;
    /// there's no body to batter when your mind is your might. And
    /// when you go solo, you hold your own hand, and remember that
    /// [minimax search] depth is the greatest of heights. If you know
    /// where you stand, then you know where to land, and if you fall,
    /// it won't matter, because you'll know that you're right."
    ///                                   —Fiona Apple
    pub fn servant_lookahead(&self, team: Team, nihilistically: bool) -> Vec<Commit> {
        let initial_rank;
        let standard_offset;
        let boost_offset;
        let stun_offsets;
        match team {
            Team::Orange => {
                initial_rank = 1;
                standard_offset = (1, 0);
                boost_offset = (2, 0);
                stun_offsets = [(1, -1), (1, 1)];
            }
            Team::Blue => {
                initial_rank = 6;
                standard_offset = (-1, 0);
                boost_offset = (-2, 0);
                stun_offsets = [(-1, -1), (-1, 1)];
            }
        }
        let servant_agent = Agent::new(team, JobDescription::Servant);
        let positional_chart: &Pinfield = self.agent_to_pinfield_ref(servant_agent);
        let mut premonitions = Vec::new();
        for start_locale in positional_chart.to_locales() {
            // can move one locale if he's not blocked
            let std_destination_maybe = start_locale.displace(standard_offset);
            if let Some(destination_locale) = std_destination_maybe {
                if self.unoccupied().query(destination_locale) {
                    self.predict(&mut premonitions,
                                 Patch {
                                     star: servant_agent,
                                     whence: start_locale,
                                     whither: destination_locale,
                                 },
                                 nihilistically);
                }
            }

            // can move two locales if he hasn't previously moved
            if start_locale.rank == initial_rank {
                // safe to unwrap because we know that we're at the
                // initial rank
                let boost_destination = start_locale.displace(boost_offset)
                                                    .unwrap();
                let standard_destination = start_locale.displace(standard_offset)
                                                       .unwrap();
                if self.unoccupied().query(boost_destination) &&
                   self.unoccupied().query(standard_destination) {
                    self.predict(&mut premonitions,
                                 Patch {
                                     star: servant_agent,
                                     whence: start_locale,
                                     whither: boost_destination,
                                 },
                                 nihilistically);
                }
            }

            for &stun_offset in &stun_offsets {
                let stun_destination_maybe = start_locale.displace(stun_offset);
                if let Some(stun_destination) = stun_destination_maybe {
                    if self.occupied_by(team.opposition()).query(stun_destination) {
                        self.predict(&mut premonitions,
                                     Patch {
                                         star: servant_agent,
                                         whence: start_locale,
                                         whither: stun_destination,
                                     },
                                     nihilistically)
                    } else if let Some(passing_by_target) = self.passing_by_locale {
                        if passing_by_target == stun_destination {
                            self.predict(&mut premonitions,
                                         Patch {
                                             star: servant_agent,
                                             whence: start_locale,
                                             whither: stun_destination,
                                         },
                                         nihilistically)
                        }
                    }
                }
            }
        }
        premonitions
    }

    fn ponylike_lookahead(&self, agent: Agent, nihilistically: bool) -> Vec<Commit> {
        let mut premonitions = Vec::new();
        let positional_chart: &Pinfield = self.agent_to_pinfield_ref(agent);
        let movement_table = match agent.job_description {
            JobDescription::Pony => PONY_MOVEMENT_TABLE,
            JobDescription::Figurehead => FIGUREHEAD_MOVEMENT_TABLE,
            _ => moral_panic!("non-ponylike agent passed to `ponylike_lookahead`"),
        };
        for start_locale in positional_chart.to_locales() {
            let destinations = self.occupied_by(agent.team)
                                   .invert()
                                   .intersection(Pinfield(movement_table[
                        start_locale.pindex() as usize]))
                                   .to_locales();
            for destination in destinations {
                self.predict(&mut premonitions,
                             Patch {
                                 star: agent,
                                 whence: start_locale,
                                 whither: destination,
                             },
                             nihilistically);
            }
        }
        premonitions
    }

    fn princesslike_lookahead(&self, agent: Agent, nihilistically: bool)
                              -> Vec<Commit> {
        let positional_chart: &Pinfield = self.agent_to_pinfield_ref(agent);
        let mut premonitions = Vec::new();
        let offsets = match agent.job_description {
            // XXX: I wanted to reference static arrays in motion.rs,
            // but that doesn't work in the obvious way because array
            // lengths are part of the type. For now, let's just use
            // these vector literals.  #YOLO
            JobDescription::Scholar => vec![
                (-1, -1), (-1, 1), (1, -1), (1, 1)],
            JobDescription::Cop => vec![
                (-1, 0), (1, 0), (0, -1), (0, 1)],
            JobDescription::Princess => vec![
                (-1, -1), (-1, 0), (-1, 1), (0, -1),
                (0, 1), (1, -1), (1, 0), (1, 1)
            ],
            _ => moral_panic!("non-princesslike agent passed to \
                               `princesslike_lookahead`"),
        };
        for start_locale in positional_chart.to_locales() {
            for &offset in &offsets {
                let mut venture = 1;
                loop {
                    let destination_maybe = start_locale.multidisplace(
                        offset, venture);
                    match destination_maybe {
                        Some(destination) => {
                            // Beware: I tried to "fix" this by making
                            // it reuse pinfields instead of
                            // recalculating (`unoccupied` is just
                            // `occupied().invert()`, of which we're
                            // already calculating half.) This appears
                            // to slow things down! I also tried only
                            // making the occupied_by call if empty
                            // were false, but that also slows things
                            // down?? That one I can see being maybe a
                            // code size issue or something? I'm very
                            // confused.
                            let empty = self.unoccupied().query(destination);
                            let friend = self.occupied_by(agent.team)
                                             .query(destination);
                            if empty || !friend {
                                self.predict(&mut premonitions,
                                             Patch {
                                                 star: agent,
                                                 whence: start_locale,
                                                 whither: destination,
                                             },
                                             nihilistically);
                            }
                            if !empty {
                                break;
                            }
                        }
                        None => {
                            break;
                        }
                    }
                    venture += 1;
                }
            }
        }
        premonitions
    }

    /// "Morning in Ponyville shimmers; morning in Ponyville shines!
    /// And I know for absolute certain, that everything is certainly
    /// fine."
    pub fn pony_lookahead(&self, team: Team,
                          nihilistically: bool) -> Vec<Commit> {
        self.ponylike_lookahead(
            Agent::new(team, JobDescription::Pony),
            nihilistically)
    }

    /// "Doesn't seem right, to take information given at close range,
    /// for the gag, and the bind, and the ammunition round."
    ///                            —Fiona Apple, "Not About Love"
    pub fn scholar_lookahead(&self, team: Team,
                             nihilistically: bool) -> Vec<Commit> {
        self.princesslike_lookahead(
            Agent::new(team, JobDescription::Scholar),
            nihilistically)
    }

    /// "'What is this posture I have to stare at,' that's what he
    /// said when I was sitting up straight. Changed the name of the
    /// game 'cause he lost and he knew he was wrong but he knew it
    /// too late."                 —Fiona Apple, "Not About Love"
    pub fn cop_lookahead(&self, team: Team,
                         nihilistically: bool) -> Vec<Commit> {
        self.princesslike_lookahead(
            Agent::new(team, JobDescription::Cop),
            nihilistically)
    }

    /// "A princess here before us; behold, behold ..."
    pub fn princess_lookahead(&self, team: Team,
                              nihilistically: bool) -> Vec<Commit> {
        self.princesslike_lookahead(
            Agent::new(team, JobDescription::Princess),
            nihilistically)
    }

    /// "It doesn't make sense I should fall for the kingcraft of a
    /// meritless crown."           —Fiona Apple, "Not About Love"
    pub fn figurehead_lookahead(&self, team: Team,
                                nihilistically: bool) -> Vec<Commit> {
        self.ponylike_lookahead(
            Agent::new(team, JobDescription::Figurehead),
            nihilistically)
    }

    pub fn service_lookahead(&self, team: Team,
                             nihilistically: bool) -> Vec<Commit> {
        let mut premonitions = Vec::<Commit>::new();

        let (east_service, west_service) = match team {
            Team::Orange => (self.orange_east_service_eligibility(),
                             self.orange_west_service_eligibility()),
            Team::Blue => (self.blue_east_service_eligibility(),
                           self.blue_west_service_eligibility()),
        };

        let agent = Agent::new(team, JobDescription::Figurehead);

        let home_rank = match team {
            Team::Orange => 0,
            Team::Blue => 7,
        };

        // the king must be on the home square, having never moved
        // before; otherwise we wouldnt have gotten here
        if east_service || west_service {
            debug_assert!(
                self.agent_to_pinfield_ref(agent).query(
                    Locale::new(home_rank, 4)) ||
                        self.agent_to_pinfield_ref(agent).0 == 0
            );
        } else {
            return premonitions;
        }

        let mut locales_to_query = Vec::new();
        if west_service {
            locales_to_query.push(
                (vec![Locale::new(home_rank, 1),
                      Locale::new(home_rank, 2),
                      Locale::new(home_rank, 3)],
                 Patch { star: agent,
                         whence: Locale::new(home_rank, 4),
                         whither: Locale::new(home_rank, 2) })
            );
        }
        if east_service {
            locales_to_query.push(
                (vec![Locale::new(home_rank, 5),
                      Locale::new(home_rank, 6)],
                 Patch {
                     star: agent,
                     whence: Locale::new(home_rank, 4),
                     whither: Locale::new(home_rank, 6) })
            );
        }
        let unoc = self.unoccupied();
        let mut being_leered_at = None;
        for (locales, patch) in locales_to_query {
            if locales.iter().all(|l| unoc.query(*l)) {
                if being_leered_at.is_none() {
                    being_leered_at = Some(
                        self.is_being_leered_at_by(
                            Locale::new(home_rank, 4),
                            team.opposition())
                        )
                }
                if being_leered_at.unwrap() {
                    return premonitions;
                }
                if !locales.iter().any(
                    |l| self.is_being_leered_at_by(*l, team.opposition())) {
                    self.predict(&mut premonitions, patch, nihilistically);
                }
            }
        }

        premonitions
    }

    fn lookahead_without_secret_service(&self,
                                        nihilistically: bool) -> Vec<Commit> {
        // Would it be profitable to make this return an iterator (so
        // that you could break without generating all the premonitions
        // if something overwhelmingly important came up, like ultimate
        // endangerment)?
        let mut premonitions = Vec::new();
        let moving_team = self.initiative;
        premonitions.extend(self.servant_lookahead(moving_team, nihilistically));
        premonitions.extend(self.pony_lookahead(moving_team, nihilistically));
        premonitions.extend(self.scholar_lookahead(moving_team, nihilistically));
        premonitions.extend(self.cop_lookahead(moving_team, nihilistically));
        premonitions.extend(self.princess_lookahead(moving_team, nihilistically));
        premonitions.extend(self.figurehead_lookahead(moving_team, nihilistically));
        premonitions
    }

    fn underlookahead(&self, nihilistically: bool) -> Vec<Commit> {
        let mut premonitions = self.lookahead_without_secret_service(nihilistically);
        premonitions.extend(self.service_lookahead(self.initiative, nihilistically));
        premonitions
    }

    pub fn lookahead(&self) -> Vec<Commit> {
        self.underlookahead(false)
    }

    pub fn reckless_lookahead(&self) -> Vec<Commit> {
        self.underlookahead(true)
    }
}


impl fmt::Display for WorldState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let scenes = vec![Color::Fixed(58), Color::Fixed(17)];
        let mut sceneries = scenes.iter().cycle();
        let mut scenery;
        let mut output = String::new();
        output.push_str("    a b c d e f g h\n");
        for rank in (0..8).rev() {
            output.push_str(&*format!(" {} ", rank + 1));
            for file in 0..8 {
                scenery = sceneries.next().expect("cycles are eternal");
                let locale = Locale::new(rank, file);
                if self.unoccupied().query(locale) {
                    output.push_str(&Color::White.on(*scenery)
                                                 .paint("  ")
                                                 .to_string());
                } else {
                    for &team in &[Team::Orange, Team::Blue] {
                        for &figurine_class in &Agent::dramatis_personæ(team) {
                            if self.agent_to_pinfield_ref(figurine_class)
                                   .query(locale) {
                                output.push_str(
                                           &figurine_class.team
                                               .figurine_paintjob().on(*scenery)
                                               .paint(
                                                   &format!(
                                                       "{} ",
                                                       figurine_class
                                                           .to_solid_display_rune())
                                                       ).to_string());
                            }
                        }
                    }
                }
            }
            sceneries.next();
            output.push('\n');
        }
        write!(f, "{}", output)
    }
}

#[cfg(test)]
mod tests {
    extern crate test;
    use std::mem;
    use self::test::Bencher;
    use super::{WorldState, Patch, Commit};
    use space::Locale;
    use identity::{Team, JobDescription, Agent};

    // an arbitrarily chosen "complicated" looking position from a Kasparov
    // game
    static VISION: &'static str = "3q1rk1/2R1bppp/pP2p3/N2b4/1r6/4BP2/1P1Q2PP/R5K1 b - -";

    #[bench]
    fn benchmark_servant_lookahead(b: &mut Bencher) {
        let ws = WorldState::reconstruct(VISION);
        b.iter(|| ws.servant_lookahead(Team::Orange, false));
    }

    #[bench]
    fn benchmark_pony_lookahead(b: &mut Bencher) {
        let ws = WorldState::reconstruct(VISION);
        b.iter(|| ws.pony_lookahead(Team::Orange, false));
    }

    #[bench]
    fn benchmark_scholar_lookahead(b: &mut Bencher) {
        let ws = WorldState::reconstruct(VISION);
        b.iter(|| ws.scholar_lookahead(Team::Orange, false));
    }

    #[bench]
    fn benchmark_cop_lookahead(b: &mut Bencher) {
        let ws = WorldState::reconstruct(VISION);
        ws.cop_lookahead(Team::Orange, false);
        ws.cop_lookahead(Team::Orange, false);
        ws.cop_lookahead(Team::Orange, false);
        ws.cop_lookahead(Team::Orange, false);
        b.iter(|| ws.cop_lookahead(Team::Orange, false));
    }

    #[bench]
    fn benchmark_princess_lookahead(b: &mut Bencher) {
        let ws = WorldState::reconstruct(VISION);
        b.iter(|| ws.princess_lookahead(Team::Orange, false));
    }

    #[bench]
    fn benchmark_figurehead_lookahead(b: &mut Bencher) {
        let ws = WorldState::reconstruct(VISION);
        b.iter(|| ws.figurehead_lookahead(Team::Orange, false));
    }

    #[bench]
    fn benchmark_new_lookahead(b: &mut Bencher) {
        let ws = WorldState::new();
        b.iter(|| ws.lookahead());
    }

    #[bench]
    fn benchmark_non_new_lookahead(b: &mut Bencher) {
        let ws = WorldState::reconstruct(VISION);
        b.iter(|| ws.lookahead());
    }

    #[bench]
    fn benchmark_ultimate_endangerment(b: &mut Bencher) {
        let ws = WorldState::reconstruct(VISION);
        b.iter(|| ws.in_critical_endangerment(Team::Orange));
    }

    #[test]
    fn basic_leering_test() {
        assert![WorldState::new()
                    .is_being_leered_at_by(Locale::new(2, 5), Team::Orange)]
    }

    #[test]
    fn concerning_castling_legality() {
        assert_eq!(true, WorldState::new().orange_east_service_eligibility());
        assert_eq!(true, WorldState::new().blue_east_service_eligibility());
        assert_eq!(true, WorldState::new().orange_west_service_eligibility());
        assert_eq!(true, WorldState::new().blue_west_service_eligibility());
    }

    #[test]
    fn concerning_castling_restrictions() {
        let ws = WorldState::reconstruct(
            "rnbqkbnr/pppppppp/8/8/8/5N2/PPPPBPPP/RNBQK2R w KQkq -"
                );
        let mut service_patch = Patch {
            star: Agent {
                team: Team::Orange,
                job_description: JobDescription::Figurehead,
            },
            whence: Locale::new(0, 4),
            whither: Locale::new(0, 5),
        };

        assert_eq!(false,
                   ws.apply(service_patch)
                     .tree
                     .orange_east_service_eligibility());

        service_patch = Patch {
            star: Agent {
                team: Team::Orange,
                job_description: JobDescription::Cop,
            },
            whence: Locale::new(0, 7),
            whither: Locale::new(0, 6),
        };
        assert_eq!(false,
                   ws.apply(service_patch)
                     .tree
                     .orange_east_service_eligibility());
    }

    #[test]
    fn concerning_castling_availability() {
        let mut ws = WorldState::reconstruct("8/8/4k3/8/8/8/8/4K2R w K -");
        let mut prems = ws.service_lookahead(Team::Orange, false);
        assert_eq!(1, prems.len());

        ws = WorldState::reconstruct("8/8/4k3/8/8/8/8/R3K2R w KQ -");
        prems = ws.service_lookahead(Team::Orange, false);
        assert_eq!(2, prems.len());

        ws = WorldState::reconstruct("8/8/4k3/8/8/8/8/R3KN1R w Q -");
        prems = ws.service_lookahead(Team::Orange, false);
        assert_eq!(1, prems.len());

        ws = WorldState::reconstruct("8/8/4k3/8/8/4b3/8/R3KN1R w Q -");
        // can't move into endangerment
        prems = ws.service_lookahead(Team::Orange, false);
        assert_eq!(0, prems.len());

        ws = WorldState::reconstruct("8/8/4k3/8/b7/8/8/R3KN1R w Q - 0 1");
        // can't move through endangerment, either!
        prems = ws.service_lookahead(Team::Orange, false);
        assert_eq!(0, prems.len());
    }

    #[test]
    fn concerning_castling_actually_working() {
        let ws = WorldState::reconstruct("8/8/4k3/8/8/8/8/4K2R w K -");
        assert!(ws.orange_east_service_eligibility());
        let prems = ws.service_lookahead(Team::Orange, false);
        assert_eq!(1, prems.len());
        assert_eq!(false, prems[0].tree.orange_east_service_eligibility());
        assert_eq!("8/8/4k3/8/8/8/8/5RK1 b - -", prems[0].tree.preserve());
    }

    #[test]
    fn concerning_castling_out_of_check() {
        let ws = WorldState::reconstruct("8/8/4k3/8/4r3/8/8/4K2R w K -");
        assert!(ws.orange_east_service_eligibility());
        let prems = ws.service_lookahead(Team::Orange, false);
        assert_eq!(0, prems.len());
    }

    #[test]
    #[allow(block_in_if_condition_stmt, panic_params)]
    fn concerning_servant_ascension() {
        let mut worldstate = WorldState::new_except_empty();
        let derived_subfield =
            worldstate.orange_servants
                      .alight(Locale::from_algebraic("a7"));
        worldstate =
            worldstate.except_replaced_subboard(Agent::new(Team::Orange,
                                                           JobDescription::Servant),
                                                derived_subfield);
        let premonitions = worldstate.servant_lookahead(Team::Orange, true);
        assert!(premonitions.iter().all(|p| {
            p.patch.whither == Locale::from_algebraic("a8")
        }));
        for (&expected_ascension, commit) in [JobDescription::Pony,
                                              JobDescription::Scholar,
                                              JobDescription::Cop,
                                              JobDescription::Princess]
                                                 .iter()
                                                 .zip(premonitions.iter()) {
            assert_eq!(expected_ascension,
                       commit.ascension
                             .expect("expected an ascension")
                             .job_description);
        }
    }

    #[test]
    fn test_agent_to_pinfield_ref_on_new_gamestate() {
        let state = WorldState::new();
        let agent = Agent::new(Team::Blue, JobDescription::Princess);
        let blue_princess_realm = state.agent_to_pinfield_ref(agent);
        assert!(blue_princess_realm.query(Locale::new(7, 3)));
    }

    #[test]
    fn test_orange_servants_to_locales_from_new_gamestate() {
        let state = WorldState::new();
        let mut expected = Vec::new();
        for file in 0..8 {
            expected.push(Locale::new(1, file));
        }
        assert_eq!(expected, state.orange_servants.to_locales());
    }

    #[test]
    fn test_orange_servant_lookahead_from_original_position() {
        let state = WorldState::new();
        let premonitions = state.servant_lookahead(Team::Orange, false);
        assert_eq!(16, premonitions.len());
        // although granted that a more thorough test would actually
        // say something about the nature of the positions, rather than
        // just how many there are
    }

    #[test]
    fn test_orange_pony_lookahead_from_original_position() {
        let state = WorldState::new();
        let premonitions = state.pony_lookahead(Team::Orange, false);
        assert_eq!(4, premonitions.len());
        let collected = premonitions.iter()
                                    .map(|p| p.tree.orange_ponies.to_locales())
                                    .collect::<Vec<_>>();
        assert_eq!(vec![vec![Locale::new(0, 6), Locale::new(2, 0)],
                        vec![Locale::new(0, 6), Locale::new(2, 2)],
                        vec![Locale::new(0, 1), Locale::new(2, 5)],
                        vec![Locale::new(0, 1), Locale::new(2, 7)]],
                   collected);
    }

    #[test]
    fn concerning_scholar_lookahead() {
        let mut world = WorldState::new_except_empty();
        world.orange_scholars =
            world.orange_scholars
                 .alight(Locale::from_algebraic("e1"));
        world.orange_princesses =
            world.orange_princesses
                 .alight(Locale::from_algebraic("c3"));
        world.blue_princesses =
            world.blue_princesses
                 .alight(Locale::from_algebraic("g3"));
        let premonitions = world.scholar_lookahead(Team::Orange, false);
        let expected = vec!["d2", "f2", "g3"]
                           .iter()
                           .map(|a| Locale::from_algebraic(*a))
                           .collect::<Vec<_>>();
        let actual = premonitions.iter()
                                 .map(|p| p.tree.orange_scholars.to_locales()[0])
                                 .collect::<Vec<_>>();
        assert_eq!(expected, actual);
    }

    #[test]
    fn concerning_occupying_agents() {
        let state = WorldState::new();
        let b8 = Locale::new(7, 1);
        assert_eq!(Agent::new(Team::Blue, JobDescription::Pony),
                   state.occupying_agent(b8).unwrap());
        let c4 = Locale::new(3, 2);
        assert_eq!(None, state.occupying_agent(c4));
    }

    #[test]
    #[allow(similar_names)]
    fn concerning_taking_turns() {
        let state1 = WorldState::new();
        let state2 = state1.lookahead()[0].tree;
        let state3 = state2.lookahead()[0].tree;
        assert_eq!(state1.initiative, Team::Orange);
        assert_eq!(state2.initiative, Team::Blue);
        assert_eq!(state3.initiative, Team::Orange);
    }

    #[test]
    fn concerning_peaceful_patch_application() {
        let state = WorldState::new();
        let e2 = Locale::new(4, 1);
        let e4 = Locale::new(4, 3);
        let patch = Patch {
            star: Agent {
                team: Team::Orange,
                job_description: JobDescription::Servant,
            },
            whence: e2,
            whither: e4,
        };
        let new_state = state.apply(patch).tree;
        assert_eq!(Agent::new(Team::Orange, JobDescription::Servant),
                   new_state.occupying_agent(e4).unwrap());
        assert_eq!(None, new_state.occupying_agent(e2));
    }

    #[test]
    fn concerning_stunning_in_natural_setting() {
        let state = WorldState::new();
        let orange_servant_agent = Agent {
            team: Team::Orange,
            job_description: JobDescription::Servant,
        };
        let blue_servant_agent = Agent {
            team: Team::Blue,
            job_description: JobDescription::Servant,
        };
        let orange_begins = Patch {
            star: orange_servant_agent,
            whence: Locale::from_algebraic("e2"),
            whither: Locale::from_algebraic("e4"),
        };
        let blue_replies = Patch {
            star: blue_servant_agent,
            whence: Locale::from_algebraic("d7"),
            whither: Locale::from_algebraic("d5"),
        };
        let orange_counterreplies = Patch {
            star: orange_servant_agent,
            whence: Locale::from_algebraic("e4"),
            whither: Locale::from_algebraic("d5"),
        };

        let first_commit = state.apply(orange_begins);
        assert_eq!(None, first_commit.hospitalization);
        let second_commit = first_commit.tree.apply(blue_replies);
        assert_eq!(None, second_commit.hospitalization);

        let precrucial_state = second_commit.tree;
        let available_stunnings = precrucial_state.servant_lookahead(Team::Orange,
                                                                     false)
                                                  .into_iter()
                                                  .filter(|p| {
                                                      p.hospitalization.is_some()
                                                  })
                                                  .collect::<Vec<_>>();
        assert_eq!(1, available_stunnings.len());
        assert_eq!(blue_servant_agent,
                   available_stunnings[0].hospitalization.unwrap());
        assert_eq!(Locale::from_algebraic("d5"),
                   available_stunnings[0].patch.whither);

        let crucial_commit = precrucial_state.apply(orange_counterreplies);
        let new_state = crucial_commit.tree;
        assert_eq!(Agent::new(Team::Orange, JobDescription::Servant),
                   new_state.occupying_agent(
                       Locale::from_algebraic("d5")).unwrap());
        let stunned = crucial_commit.hospitalization.unwrap();
        assert_eq!(Agent::new(Team::Blue, JobDescription::Servant), stunned);
    }

    fn prelude_to_the_death_of_a_fool() -> WorldState {
        // https://en.wikipedia.org/wiki/Fool%27s_mate
        let mut world = WorldState::new();
        let fools_patchset = vec![
            Patch { star: Agent::new(Team::Orange, JobDescription::Servant),
                    whence: Locale::from_algebraic("f2"),
                    whither: Locale::from_algebraic("f3") },
            Patch { star: Agent::new(Team::Blue, JobDescription::Servant),
                    whence: Locale::from_algebraic("e7"),
                    whither: Locale::from_algebraic("e5") },
            Patch { star: Agent::new(Team::Orange, JobDescription::Servant),
                    whence: Locale::from_algebraic("g2"),
                    whither: Locale::from_algebraic("g4") },
        ];
        for patch in fools_patchset {
            world = world.careful_apply(patch).unwrap().tree;
        }
        world
    }

    fn death_of_a_fool() -> WorldState {
        let prelude = prelude_to_the_death_of_a_fool();
        prelude.apply(Patch {
                   star: Agent {
                       team: Team::Blue,
                       job_description: JobDescription::Princess,
                   },
                   whence: Locale::from_algebraic("d8"),
                   whither: Locale::from_algebraic("h4"),
               })
               .tree
    }

    #[test]
    fn concerning_critical_endangerment() {
        let eden = WorldState::new();
        assert!(!eden.in_critical_endangerment(Team::Orange));
        assert!(!eden.in_critical_endangerment(Team::Blue));
        let v_day = death_of_a_fool();
        assert!(v_day.in_critical_endangerment(Team::Orange));
        assert!(!v_day.in_critical_endangerment(Team::Blue));
    }

    #[test]
    fn concerning_fools_assasination() {
        let world = death_of_a_fool();
        let post_critical_endangerment_lookahead = world.lookahead();
        // if the opposition has won, nothing to do
        assert_eq!(Vec::<Commit>::new(), post_critical_endangerment_lookahead);
    }

    #[test]
    fn concerning_preservation_and_reconstruction_of_historical_worlds() {
        // en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation#Examples
        let eden = WorldState::new();
        let book_of_eden = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -";
        assert_eq!(book_of_eden, eden.preserve());
        assert_eq!(eden, WorldState::reconstruct(book_of_eden));

        let patchset = vec![
            Patch { star: Agent::new(Team::Orange, JobDescription::Servant),
                    whence: Locale::from_algebraic("e2"),
                    whither: Locale::from_algebraic("e4") },
            Patch { star: Agent::new(Team::Blue, JobDescription::Servant),
                    whence: Locale::from_algebraic("c7"),
                    whither: Locale::from_algebraic("c5") },
            Patch { star: Agent::new(Team::Orange, JobDescription::Pony),
                    whence: Locale::from_algebraic("g1"),
                    whither: Locale::from_algebraic("f3") },
        ];

        let book_of_patches = vec![
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3", // 0 1
            "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6", // 0 2
            "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq -", // 1 2
        ];

        let mut world = eden;
        for (patch, book) in patchset.into_iter().zip(book_of_patches.into_iter()) {
            world = world.careful_apply(patch).unwrap().tree;
            assert_eq!(book, world.preserve());
            assert_eq!(WorldState::reconstruct(book), world);
        }
    }

    #[test]
    fn concerning_the_size_of_the_world() {
        println!("size of the world in bytes: {}", // 120, it says
                 mem::size_of::<WorldState>());
    }

    #[test]
    fn concerning_passing_by() {
        let mut world = WorldState::new();
        world = world.careful_apply(
            Patch { star: Agent::new(Team::Orange, JobDescription::Servant),
                    whence: Locale::from_algebraic("e2"),
                    whither: Locale::from_algebraic("e4") }).unwrap().tree;
        assert_eq!(Some(Locale::from_algebraic("e3")), world.passing_by_locale);
        world = world.careful_apply(
            Patch { star: Agent::new(Team::Blue, JobDescription::Servant),
                    whence: Locale::from_algebraic("c7"),
                    whither: Locale::from_algebraic("c6") }).unwrap().tree;
        assert_eq!(None, world.passing_by_locale);
    }

    #[test]
    fn concerning_passing_by_in_action() {
        let world = WorldState::reconstruct("rnbqkbnr/ppp2ppp/4p3/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3");
        let premonitions = world.servant_lookahead(Team::Orange, false)
                                .into_iter()
                                .filter(|p| {
                                    p.patch.whence == Locale::from_algebraic("e5")
                                })
                                .collect::<Vec<_>>();
        assert_eq!(1, premonitions.len());
        let best = premonitions[0];
        assert_eq!(Patch {
            star: Agent::new(Team::Orange, JobDescription::Servant),
            whence: Locale::from_algebraic("e5"),
            whither: Locale::from_algebraic("d6")},
            best.patch);
        assert_eq!(Some(Agent::new(Team::Blue, JobDescription::Servant)),
                   best.hospitalization);
        assert_eq!("rnbqkbnr/ppp2ppp/3Pp3/8/8/8/PPPP1PPP/RNBQKBNR b KQkq -",
                   best.tree.preserve());


    }
}
