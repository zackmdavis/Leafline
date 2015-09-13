//! the `life` module of the Leafline oppositional strategy game engine
use std::fmt;

use space::{Locale, Pinfield};
use identity::{Team, JobDescription, Agent};
use motion::{PONY_MOVEMENT_TABLE, FIGUREHEAD_MOVEMENT_TABLE};


/// represents the movement of a figurine
#[derive(Eq,PartialEq,Debug,Clone,Hash,RustcEncodable,RustcDecodable)]
pub struct Patch {
    pub star: Agent,
    pub whence: Locale,
    pub whither: Locale,
    pub hospitalization: Option<Agent>,
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
}


/// represents the outcome of a team's turn with a `patch` governing
/// the figurine moved, the state of the world after the turn (`tree`),
/// and whether an opposing figurine was stunned and put in the hospital,
/// and if so, which one
#[derive(Eq,PartialEq,Debug,Clone,Hash)]
pub struct Commit {
    pub patch: Patch,
    pub tree: WorldState,
    pub ascension: Option<Agent>,
}

impl fmt::Display for Commit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let hospital_report = match self.patch.hospitalization {
            Some(stunning_victim) => format!(", stunning {}", stunning_victim),
            None => "".to_owned(),
        };
        let ascension_report = match self.ascension {
            Some(ascended_form) => {
                match ascended_form.job_description {
                    JobDescription::Servant => moral_panic!(
                        "servant purportedly 'ascending'\
                         to his own station (?!)"),
                    JobDescription::Pony => format!(
                        ", transforming into {}", ascended_form),
                    JobDescription::Cop => format!(
                        ", being brevetted to {}", ascended_form),
                    JobDescription::Scholar | JobDescription::Princess =>
                        format!(", transitioning into {}", ascended_form),
                    JobDescription::Figurehead => moral_panic!(
                        "servant ascending to figurehead"),
                }
            }
            None => "".to_owned(),
        };
        let report = hospital_report + &ascension_report;
        write!(
            f,
            "{} from {} to {}{}",
            self.patch.star,
            self.patch.whence.to_algebraic(),
            self.patch.whither.to_algebraic(),
            report
        )
    }
}


#[derive(Eq,PartialEq,Debug,Copy,Clone,Hash)]
pub struct WorldState {
    pub to_move: Team,

    pub orange_servants: Pinfield,
    pub orange_ponies: Pinfield,
    pub orange_scholars: Pinfield,
    pub orange_cops: Pinfield,
    pub orange_princesses: Pinfield,
    pub orange_figurehead: Pinfield,
    pub orange_east_service_eligibility: bool,
    pub orange_west_service_eligibility: bool,

    pub blue_servants: Pinfield,
    pub blue_ponies: Pinfield,
    pub blue_scholars: Pinfield,
    pub blue_cops: Pinfield,
    pub blue_princesses: Pinfield,
    pub blue_figurehead: Pinfield,
    pub blue_east_service_eligibility: bool,
    pub blue_west_service_eligibility: bool,
}

const ORANGE_FIGUREHEAD_START: Locale = Locale { rank: 0, file: 4 };
const BLUE_FIGUREHEAD_START: Locale = Locale { rank: 7, file: 4 };

impl WorldState {
    pub fn new() -> Self {
        let mut orange_servant_locales = Vec::new();
        let mut blue_servant_locales = Vec::new();
        for f in 0..8 {
            orange_servant_locales.push(Locale { rank: 1, file: f });
            blue_servant_locales.push(Locale { rank: 6, file: f });
        }
        WorldState {
            to_move: Team::Orange,

            orange_servants: Pinfield::init(&orange_servant_locales),
            orange_ponies: Pinfield::init(&vec![Locale { rank: 0, file: 1 },
                      Locale { rank: 0, file: 6 }]),
            orange_scholars: Pinfield::init(&vec![Locale { rank: 0, file: 2 },
                      Locale { rank: 0, file: 5 }]),
            orange_cops: Pinfield::init(&vec![Locale { rank: 0, file: 0 },
                      Locale { rank: 0, file: 7 }]),
            orange_princesses: Pinfield::init(
                &vec![Locale { rank: 0, file: 3 }]),
            orange_figurehead: Pinfield::init(&vec![ORANGE_FIGUREHEAD_START]),
            orange_east_service_eligibility: true,
            orange_west_service_eligibility: true,

            blue_servants: Pinfield::init(&blue_servant_locales),
            blue_ponies: Pinfield::init(&vec![Locale { rank: 7, file: 1 },
                      Locale { rank: 7, file: 6 }]),
            blue_scholars: Pinfield::init(&vec![Locale { rank: 7, file: 2 },
                      Locale { rank: 7, file: 5 }]),
            blue_cops: Pinfield::init(&vec![Locale { rank: 7, file: 0 },
                      Locale { rank: 7, file: 7 }]),
            blue_princesses: Pinfield::init(&vec![Locale { rank: 7, file: 3 }]),
            blue_figurehead: Pinfield::init(&vec![BLUE_FIGUREHEAD_START]),
            blue_east_service_eligibility: true,
            blue_west_service_eligibility: true,
        }
    }

    // XXX: for API consistency with Pinfield, maybe this should be
    // `new` and the current `new` should be `init`??
    pub fn new_except_empty() -> Self {
        WorldState {
            to_move: Team::Orange,

            orange_servants: Pinfield::new(),
            orange_ponies: Pinfield::new(),
            orange_scholars: Pinfield::new(),
            orange_cops: Pinfield::new(),
            orange_princesses: Pinfield::new(),
            orange_figurehead: Pinfield::new(),
            orange_east_service_eligibility: false,
            orange_west_service_eligibility: false,

            blue_servants: Pinfield::new(),
            blue_ponies: Pinfield::new(),
            blue_scholars: Pinfield::new(),
            blue_cops: Pinfield::new(),
            blue_princesses: Pinfield::new(),
            blue_figurehead: Pinfield::new(),
            blue_east_service_eligibility: false,
            blue_west_service_eligibility: false,
        }
    }

    pub fn cede_initiative(&mut self) {
        self.to_move = self.to_move.opposition();
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
        let agent = Agent { team: team.opposition(),
                            job_description: JobDescription::Figurehead };
        let pinfield = self.agent_to_pinfield_ref(agent);
        let mut tree = self.except_replaced_subboard(
            agent, pinfield.alight(locale));
        tree.to_move = team;
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
                    Locale { rank: rank, file: file });
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
        let to_move_indication_rune = match self.to_move {
            // TODO: think of some remotely plausible rationalization for 'w'
            Team::Orange => 'w',
            Team::Blue => 'b',
        };
        book.push(' ');
        book.push(to_move_indication_rune);
        book.push(' ');
        let mut any_service = false;
        for &(service_eligibility, eligibility_rune) in &[
            (self.orange_east_service_eligibility, 'K'),
            (self.orange_west_service_eligibility, 'Q'),
            (self.blue_east_service_eligibility, 'k'),
            (self.blue_west_service_eligibility, 'q')] {
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
        book
    }

    pub fn reconstruct(scan: String) -> Self {
        let mut rank = 7;
        let mut file = 0;
        let mut world = WorldState::new_except_empty();
        let mut volumes = scan.split(' ');
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
                // XXX "If you use `@` with `|`, you need to make sure
                // the name is bound in each part of the pattern" oh like
                // that's ergonomic
                r @ 'P' |
                r @ 'N' |
                r @ 'B' |
                r @ 'R' |
                r @ 'Q' |
                r @ 'K' |
                r @ 'p' |
                r @ 'n' |
                r @ 'b' |
                r @ 'r' |
                r @ 'q' |
                r @ 'k' => {
                    let agent = Agent::from_preservation_rune(r);
                    let derived_pinfield;
                    {
                        let hot_pinfield = world.agent_to_pinfield_ref(agent);
                        derived_pinfield = hot_pinfield.alight(
                            Locale { rank: rank, file: file });
                    }
                    // XXX: this isn't Clojure; copying a
                    // datastructure with a small diff actually has costs
                    world = world.except_replaced_subboard(
                        agent, derived_pinfield);
                    file += 1;
                }
                r => moral_panic!(format!("Unexpected rune '{}'", r)),
            }
        }
        let rune_of_those_with_initiative = volumes.next()
                                                   .unwrap()
                                                   .chars()
                                                   .next()
                                                   .unwrap();
        world.to_move = match rune_of_those_with_initiative {
            'w' => Team::Orange,
            'b' => Team::Blue,
            _ => moral_panic!("Non-initiative-preserving-rune passed to \
                               a match expecting such"),
        };
        let secret_service_eligibilities = volumes.next().unwrap();
        for eligibility in secret_service_eligibilities.chars() {
            match eligibility {
                'K' => {
                    world.orange_east_service_eligibility = true;
                }
                'Q' => {
                    world.orange_west_service_eligibility = true;
                }
                'k' => {
                    world.blue_east_service_eligibility = true;
                }
                'q' => {
                    world.blue_west_service_eligibility = true;
                }
                '-' => {
                    break;
                }
                r => {
                    moral_panic!(
                        format!("non-eligibility rune '{}'", r)
                    );
                }
            }
        }
        world
    }

    pub fn except_replaced_subboard(&self, for_whom: Agent,
                                    subboard: Pinfield) -> Self {
        let mut resultant_state = self.clone();
        resultant_state.agent_to_pinfield_mutref(for_whom).0 = subboard.0;
        resultant_state
    }

    pub fn occupied_by(&self, team: Team) -> Pinfield {
        match team {
            Team::Orange => self.orange_servants
                                .union(self.orange_ponies)
                                .union(self.orange_scholars)
                                .union(self.orange_cops)
                                .union(self.orange_princesses)
                                .union(self.orange_figurehead),
            Team::Blue => self.blue_servants
                              .union(self.blue_ponies)
                              .union(self.blue_scholars)
                              .union(self.blue_cops)
                              .union(self.blue_princesses)
                              .union(self.blue_figurehead),
        }
    }

    pub fn occupied(&self) -> Pinfield {
        self.occupied_by(Team::Orange).union(self.occupied_by(Team::Blue))
    }

    pub fn unoccupied(&self) -> Pinfield {
        self.occupied().invert()
    }

    pub fn occupying_affiliated_agent(
            &self, at: Locale, team: Team) -> Option<Agent> {
        for agent in Agent::dramatis_personæ(team).into_iter() {
            if self.agent_to_pinfield_ref(agent).query(at) {
                return Some(agent)
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
        let mut tree = self.except_replaced_subboard(patch.star,
                                                     derived_subboard);
        match patch.star.job_description {
            JobDescription::Figurehead => {
                match patch.star.team {
                    Team::Orange => {
                        tree.orange_east_service_eligibility = false;
                        tree.orange_west_service_eligibility = false;
                    }
                    Team::Blue => {
                        tree.blue_east_service_eligibility = false;
                        tree.blue_west_service_eligibility = false;
                    }
                }
            }
            JobDescription::Cop => {
                match (patch.whence.file, patch.star.team) {
                    (0, Team::Orange) => {
                        tree.orange_west_service_eligibility = false;
                    }
                    (7, Team::Orange) => {
                        tree.orange_east_service_eligibility = false;
                    }
                    (0, Team::Blue) => {
                        tree.blue_west_service_eligibility = false;
                    }
                    (7, Team::Blue) => {
                        tree.blue_east_service_eligibility = false;
                    }
                    _ => {}
                }
            }
            _ => {}
        }


        if patch.concerns_secret_service() {
            let cop_agent = Agent { team: patch.star.team,
                                    job_description: JobDescription::Cop };
            let (start_file, end_file) = match patch.whither.file {
                6 => (7, 5),
                2 => (0, 3),
                _ => moral_panic!("This looked like a Secret Service commit, \
                                   but it is not"),
            };

            let secret_derived_subboard = tree.agent_to_pinfield_ref(
                cop_agent)
                .transit(
                    Locale { rank: patch.whither.rank, file: start_file, },
                    Locale { rank: patch.whither.rank, file: end_file, });
            tree = tree.except_replaced_subboard(cop_agent,
                                                 secret_derived_subboard);
        }
        if let Some(stunned) = patch.hospitalization {
            // if someone was stunned, put her or him in the hospital
            let further_derived_subboard = tree.agent_to_pinfield_ref(stunned)
                                               .quench(patch.whither);
            tree = tree.except_replaced_subboard(
                stunned, further_derived_subboard);
        }
        tree.cede_initiative();
        Commit { patch: patch, tree: tree, ascension: None }
    }

    pub fn in_critical_endangerment(&self, team: Team) -> bool {
        let mut contingency = *self;
        contingency.to_move = team.opposition();
        let premonitions = contingency.reckless_lookahead();
        for premonition in &premonitions {
            if let Some(patient) = premonition.patch.hospitalization {
                if patient.job_description == JobDescription::Figurehead {
                    return true;
                }
            }
        }
        false
    }

    pub fn careful_apply(&self, patch: Patch) -> Option<Commit> {
        let force_commit = self.apply(patch);
        if force_commit.tree.in_critical_endangerment(self.to_move) {
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
                let mut ascendency = premonition.clone();
                ascendency.ascension = Some(ascended);
                let vessel_pinfield = premonition.tree.agent_to_pinfield_ref(
                    premonition.patch.star).quench(premonition.patch.whither);
                let ascended_pinfield = premonition.tree
                    .agent_to_pinfield_ref(ascended)
                    .alight(premonition.patch.whither);
                ascendency.tree = ascendency.tree
                    .except_replaced_subboard(
                        premonition.patch.star, vessel_pinfield)
                    .except_replaced_subboard(
                        ascended, ascended_pinfield);
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

    /// generate possible commits for servants of the given team
    pub fn servant_lookahead(&self, team: Team,
                             nihilistically: bool) -> Vec<Commit> {
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
        let servant_agent = Agent { team: team,
                                    job_description: JobDescription::Servant };
        let positional_chart: &Pinfield = self.agent_to_pinfield_ref(
            servant_agent);
        let mut premonitions = Vec::new();
        for start_locale in positional_chart.to_locales().into_iter() {
            // can move one locale if he's not blocked
            let std_destination_maybe = start_locale.displace(standard_offset);
            if let Some(destination_locale) = std_destination_maybe {
                if self.unoccupied().query(destination_locale) {
                    self.predict(&mut premonitions,
                                 Patch {
                                     star: servant_agent,
                                     whence: start_locale,
                                     whither: destination_locale,
                                     hospitalization: None,
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
                                     hospitalization: None,
                                 },
                                 nihilistically);
                }
            }

            for &stun_offset in &stun_offsets {
                let stun_destination_maybe = start_locale.displace(stun_offset);
                if let Some(stun_destination) = stun_destination_maybe {
                    if self.occupied_by(team.opposition()).query(
                            stun_destination) {
                        let some_patient = self.occupying_affiliated_agent(
                            stun_destination, team.opposition());
                        self.predict(&mut premonitions,
                                     Patch {
                                         star: servant_agent,
                                         whence: start_locale,
                                         whither: stun_destination,
                                         hospitalization: some_patient,
                                     },
                                     nihilistically)
                    }
                }
            }
        }
        premonitions
    }

    fn ponylike_lookahead(&self, agent: Agent,
                          nihilistically: bool) -> Vec<Commit> {
        let mut premonitions = Vec::new();
        let positional_chart: &Pinfield = self.agent_to_pinfield_ref(agent);
        let movement_table = match agent.job_description {
            JobDescription::Pony => PONY_MOVEMENT_TABLE,
            JobDescription::Figurehead => FIGUREHEAD_MOVEMENT_TABLE,
            _ => moral_panic!("non-ponylike agent passed to \
                               `ponylike_lookahead`"),
        };
        for start_locale in positional_chart.to_locales().into_iter() {
            let possible_destinations = Pinfield(
                movement_table[start_locale.pindex() as usize]);
            let empty_destinations = self.unoccupied()
                .intersection(possible_destinations);
            let empty_locales = empty_destinations.to_locales();
            for &empty_locale in &empty_locales {
                self.predict(&mut premonitions,
                             Patch { star: agent,
                                     whence: start_locale,
                                     whither: empty_locale,
                                     hospitalization: None },
                             nihilistically);
            }
            let occupied_locales = possible_destinations
                .difference(empty_destinations).to_locales();
            for &occupied_locale in &occupied_locales {
                let some_patient = self.occupying_affiliated_agent(
                    occupied_locale, agent.team.opposition());
                if some_patient.is_some() {
                    self.predict(&mut premonitions,
                                 Patch { star: agent,
                                         whence: start_locale,
                                         whither: occupied_locale,
                                         hospitalization: some_patient },
                                 nihilistically);
                }
            }
            // let stun_opportunities = self.occupied_by(agent.team.opposition())
            //     .intersection(possible_destinations)
            //     .to_locales();
            // for &stun_opportunity in &stun_opportunities {
            //     let some_patient = self.occupying_affiliated_agent(
            //         stun_opportunity, agent.team.opposition());
            //     self.predict(&mut premonitions,
            //                  Patch { star: agent,
            //                          whence: start_locale,
            //                          whither: stun_opportunity,
            //                          hospitalization: some_patient },
            //                  nihilistically);
            // }
        }
        premonitions
    }

    fn princesslike_lookahead(&self, agent: Agent,
                              nihilistically: bool) -> Vec<Commit> {
        let positional_chart: &Pinfield = self.agent_to_pinfield_ref(agent);
        let mut premonitions = Vec::new();
        let offsets = match agent.job_description {
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
        for start_locale in positional_chart.to_locales().into_iter() {
            for &offset in &offsets {
                let mut venture = 1;
                loop {
                    let destination_maybe = start_locale.multidisplace(
                        offset, venture);
                    match destination_maybe {
                        Some(destination) => {
                            let empty = self.unoccupied().query(destination);
                            let friend = self.occupied_by(agent.team)
                                             .query(destination);
                            if empty {
                                self.predict(&mut premonitions,
                                             Patch {
                                                 star: agent,
                                                 whence: start_locale,
                                                 whither: destination,
                                                 hospitalization: None,
                                             },
                                             nihilistically);
                            } else if !friend {
                                let some_patient = self
                                    .occupying_affiliated_agent(
                                        destination, agent.team.opposition());
                                self.predict(&mut premonitions,
                                             Patch {
                                                 star: agent,
                                                 whence: start_locale,
                                                 whither: destination,
                                                 hospitalization: some_patient,
                                             },
                                             nihilistically);
                                break;
                            }
                            else {
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

    pub fn pony_lookahead(&self, team: Team,
                          nihilistically: bool) -> Vec<Commit> {
        self.ponylike_lookahead(
            Agent { team: team, job_description: JobDescription::Pony },
            nihilistically)
    }

    pub fn scholar_lookahead(&self, team: Team,
                             nihilistically: bool) -> Vec<Commit> {
        self.princesslike_lookahead(
            Agent { team: team, job_description: JobDescription::Scholar },
            nihilistically)
    }

    pub fn cop_lookahead(&self, team: Team,
                         nihilistically: bool) -> Vec<Commit> {
        self.princesslike_lookahead(
            Agent { team: team, job_description: JobDescription::Cop },
            nihilistically)
    }

    pub fn princess_lookahead(&self, team: Team,
                              nihilistically: bool) -> Vec<Commit> {
        self.princesslike_lookahead(
            Agent { team: team, job_description: JobDescription::Princess },
            nihilistically)
    }

    pub fn figurehead_lookahead(&self, team: Team,
                                nihilistically: bool) -> Vec<Commit> {
        self.ponylike_lookahead(
            Agent { team: team, job_description: JobDescription::Figurehead },
            nihilistically)
    }

    pub fn service_lookahead(&self, team: Team,
                             nihilistically: bool) -> Vec<Commit> {
        let mut premonitions = Vec::<Commit>::new();

        let (east_service, west_service) = match team {
            Team::Orange => (self.orange_east_service_eligibility,
                             self.orange_west_service_eligibility),
            Team::Blue => (self.blue_east_service_eligibility,
                           self.blue_west_service_eligibility),
        };

        let agent = Agent { team: team,
                            job_description: JobDescription::Figurehead };

        let home_rank = match team {
            Team::Orange => 0,
            Team::Blue => 7,
        };

        // the king must be on the home square, having never moved
        // before; otherwise we wouldnt have gotten here
        if east_service || west_service {
            debug_assert!(
                self.agent_to_pinfield_ref(agent).query(
                    Locale { rank: home_rank, file: 4 }) ||
                        self.agent_to_pinfield_ref(agent).0 == 0
            );
        } else {
            return premonitions;
        }

        let mut locales_to_query = Vec::new();
        if west_service {
            locales_to_query.push(
                (vec![Locale { rank: home_rank, file: 1 },
                      Locale { rank: home_rank, file: 2 },
                      Locale { rank: home_rank, file: 3 }],
                 Patch { star: agent,
                         whence: Locale { rank: home_rank, file: 4 },
                         whither: Locale { rank: home_rank, file: 2 },
                         hospitalization: None })
            );
        }
        if east_service {
            locales_to_query.push(
                (vec![Locale { rank: home_rank, file: 5 },
                      Locale { rank: home_rank, file: 6 }],
                 Patch {
                     star: agent,
                     whence: Locale { rank: home_rank, file: 4 },
                     whither: Locale { rank: home_rank, file: 6 }, hospitalization: None })
            );
        }
        let unoc = self.unoccupied();
        let mut being_leered_at = None;
        for (locales, patch) in locales_to_query {
            if locales.iter().all(|l| unoc.query(*l)) {
                match being_leered_at {
                    None => {
                        being_leered_at = Some(
                            self.is_being_leered_at_by(
                                Locale {
                                    rank: home_rank,
                                    file: 4,
                                },
                                team.opposition())
                        )
                    },
                    _ => {}
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
        let moving_team = self.to_move;
        premonitions.extend(self.servant_lookahead(moving_team, nihilistically));
        premonitions.extend(self.pony_lookahead(moving_team, nihilistically));
        premonitions.extend(self.scholar_lookahead(moving_team, nihilistically));
        premonitions.extend(self.cop_lookahead(moving_team, nihilistically));
        premonitions.extend(self.princess_lookahead(
            moving_team, nihilistically));
        premonitions.extend(self.figurehead_lookahead(
            moving_team, nihilistically));
        premonitions
    }

    fn underlookahead(&self, nihilistically: bool) -> Vec<Commit> {
        let mut premonitions = self.lookahead_without_secret_service(
            nihilistically);
        premonitions.extend(self.service_lookahead(
            self.to_move, nihilistically));
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
        let mut output = String::new();
        output.push_str("  a b c d e f g h\n");
        for rank in (0..8).rev() {
            output.push_str(&*format!("{} ", rank+1));
            for file in 0..8 {
                let locale = Locale { rank: rank, file: file };
                if self.occupied().invert().query(locale) {
                    output.push_str(&*format!("_ "));
                } else {
                    for &team in &[Team::Orange, Team::Blue] {
                        for &figurine_class in &Agent::dramatis_personæ(team) {
                            if self.agent_to_pinfield_ref(figurine_class)
                                   .query(locale) {
                                output.push_str(
                                    &*format!("{} ", figurine_class));
                            }
                        }
                    }
                }
            }
            output.push('\n');
        }
        write!(f, "{}", output)
    }
}

#[cfg(test)]
mod tests {
    extern crate test;
    use self::test::Bencher;
    use super::{WorldState, Patch, Commit};
    use space::Locale;
    use identity::{Team, JobDescription, Agent};

    // an arbitrarily chosen "complicated" looking position from a Kasparov
    // game
    static VISION: &'static str = "3q1rk1/2R1bppp/pP2p3/N2b4/1r6/4BP2/1P1Q2PP/R5K1 b -";

    #[ignore]
    #[bench]
    fn benchmark_servant_lookahead(b: &mut Bencher) {
        let ws = WorldState::reconstruct(VISION.to_owned());
        b.iter(|| ws.servant_lookahead(Team::Orange, false));
    }

    #[ignore]
    #[bench]
    fn benchmark_pony_lookahead(b: &mut Bencher) {
        let ws = WorldState::reconstruct(VISION.to_owned());
        b.iter(|| ws.pony_lookahead(Team::Orange, false));
    }

    #[ignore]
    #[bench]
    fn benchmark_scholar_lookahead(b: &mut Bencher) {
        let ws = WorldState::reconstruct(VISION.to_owned());
        b.iter(|| ws.scholar_lookahead(Team::Orange, false));
    }

    #[ignore]
    #[bench]
    fn benchmark_cop_lookahead(b: &mut Bencher) {
        let ws = WorldState::reconstruct(VISION.to_owned());
        ws.cop_lookahead(Team::Orange, false);
        ws.cop_lookahead(Team::Orange, false);
        ws.cop_lookahead(Team::Orange, false);
        ws.cop_lookahead(Team::Orange, false);
        b.iter(|| ws.cop_lookahead(Team::Orange, false));
    }

    #[ignore]
    #[bench]
    fn benchmark_princess_lookahead(b: &mut Bencher) {
        let ws = WorldState::reconstruct(VISION.to_owned());
        b.iter(|| ws.princess_lookahead(Team::Orange, false));
    }

    #[ignore]
    #[bench]
    fn benchmark_figurehead_lookahead(b: &mut Bencher) {
        let ws = WorldState::reconstruct(VISION.to_owned());
        b.iter(|| ws.figurehead_lookahead(Team::Orange, false));
    }

    #[ignore]
    #[bench]
    fn benchmark_new_lookahead(b: &mut Bencher) {
        let ws = WorldState::new();
        b.iter(|| ws.lookahead());
    }

    #[ignore]
    #[bench]
    fn benchmark_non_new_lookahead(b: &mut Bencher) {
        let ws = WorldState::reconstruct(VISION.to_owned());
        b.iter(|| ws.lookahead());
    }

    #[ignore]
    #[bench]
    fn benchmark_ultimate_endangerment(b: &mut Bencher) {
        let ws = WorldState::reconstruct(VISION.to_owned());
        b.iter(|| ws.in_critical_endangerment(Team::Orange));
    }

    #[test]
    fn basic_leering_test() {
        assert![WorldState::new().is_being_leered_at_by(
            Locale { rank: 2, file: 5 }, Team::Orange)]
    }

    #[test]
    fn concerning_castling_legality() {
        assert_eq!(true, WorldState::new().orange_east_service_eligibility);
        assert_eq!(true, WorldState::new().blue_east_service_eligibility);
        assert_eq!(true, WorldState::new().orange_west_service_eligibility);
        assert_eq!(true, WorldState::new().blue_west_service_eligibility);
    }

    #[test]
    fn concerning_castling_restrictions() {
        let ws = WorldState::reconstruct(
            "rnbqkbnr/pppppppp/8/8/8/5N2/PPPPBPPP/RNBQK2R w KQkq"
                .to_owned());
        let mut service_patch = Patch {
            star: Agent {
                team: Team::Orange,
                job_description: JobDescription::Figurehead,
            },
            whence: Locale { rank: 0, file: 4 },
            whither: Locale { rank: 0, file: 5 },
            hospitalization: None,
        };

        assert_eq!(false, ws.apply(
            service_patch).tree.orange_east_service_eligibility);

        service_patch = Patch {
            star: Agent {
                team: Team::Orange,
                job_description: JobDescription::Cop,
            },
            whence: Locale { rank: 0, file: 7 },
            whither: Locale { rank: 0, file: 6 },
            hospitalization: None,
        };
        assert_eq!(false, ws.apply(
            service_patch).tree.orange_east_service_eligibility);
    }

    #[test]
    fn concerning_castling_availability() {
        let mut ws = WorldState::reconstruct(
            "8/8/4k3/8/8/8/8/4K2R w K".to_owned());
        let mut prems = ws.service_lookahead(Team::Orange, false);
        assert_eq!(1, prems.len());

        ws = WorldState::reconstruct("8/8/4k3/8/8/8/8/R3K2R w KQ".to_owned());
        prems = ws.service_lookahead(Team::Orange, false);
        assert_eq!(2, prems.len());

        ws = WorldState::reconstruct("8/8/4k3/8/8/8/8/R3KN1R w Q".to_owned());
        prems = ws.service_lookahead(Team::Orange, false);
        assert_eq!(1, prems.len());

        ws = WorldState::reconstruct("8/8/4k3/8/8/4b3/8/R3KN1R w Q".to_owned());
        // can't move into endangerment
        prems = ws.service_lookahead(Team::Orange, false);
        assert_eq!(0, prems.len());

        ws = WorldState::reconstruct(
            "8/8/4k3/8/b7/8/8/R3KN1R w Q - 0 1".to_owned());
        // can't move through endangerment, either!
        prems = ws.service_lookahead(Team::Orange, false);
        assert_eq!(0, prems.len());
    }

    #[test]
    fn concerning_castling_actually_working() {
        let ws = WorldState::reconstruct("8/8/4k3/8/8/8/8/4K2R w K".to_owned());
        assert!(ws.orange_east_service_eligibility);
        let prems = ws.service_lookahead(Team::Orange, false);
        assert_eq!(1, prems.len());
        assert_eq!(false, prems[0].tree.orange_east_service_eligibility);
        assert_eq!("8/8/4k3/8/8/8/8/5RK1 b -", prems[0].tree.preserve());
    }

    #[test]
    fn concerning_castling_out_of_check() {
        let ws = WorldState::reconstruct(
            "8/8/4k3/8/4r3/8/8/4K2R w K".to_owned());
        assert!(ws.orange_east_service_eligibility);
        let prems = ws.service_lookahead(Team::Orange, false);
        assert_eq!(0, prems.len());
    }

    #[test]
    fn concerning_servant_ascension() {
        let mut worldstate = WorldState::new_except_empty();
        let derived_subfield = worldstate.orange_servants
                                         .alight(
                                             Locale::from_algebraic(
                                                 "a7".to_owned()));
        worldstate = worldstate.except_replaced_subboard(
            Agent { team: Team::Orange,
                    job_description: JobDescription::Servant },
            derived_subfield);
        let premonitions = worldstate.servant_lookahead(Team::Orange, true);
        assert!(premonitions.iter().all(
            |p| p.patch.whither == Locale::from_algebraic("a8".to_owned())));
        for (&expected_ascension, commit) in [JobDescription::Pony, JobDescription::Scholar, JobDescription::Cop, JobDescription::Princess].iter().zip(premonitions.iter()) {
            assert_eq!(expected_ascension,
                       commit.ascension.expect(
                           "expected an ascension").job_description);
        }
    }

    #[test]
    fn test_agent_to_pinfield_ref_on_new_gamestate() {
        let state = WorldState::new();
        let agent = Agent { team: Team::Blue,
                            job_description: JobDescription::Princess };
        let blue_princess_realm = state.agent_to_pinfield_ref(agent);
        assert!(blue_princess_realm.query(Locale { rank: 7, file: 3 }));
    }

    #[test]
    fn test_orange_servants_to_locales_from_new_gamestate() {
        let state = WorldState::new();
        let mut expected = Vec::new();
        for file in 0..8 {
            expected.push(Locale { rank: 1, file: file });
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
        assert_eq!(
            vec![vec![Locale { rank: 0, file: 6 },
                      Locale { rank: 2, file: 0 }],
                 vec![Locale { rank: 0, file: 6 },
                      Locale { rank: 2, file: 2 }],
                 vec![Locale { rank: 0, file: 1 },
                      Locale { rank: 2, file: 5 }],
                 vec![Locale { rank: 0, file: 1 },
                      Locale { rank: 2, file: 7 }]],
                 collected
        );
    }

    #[test]
    fn concerning_scholar_lookahead() {
        let mut world = WorldState::new_except_empty();
        world.orange_scholars = world.orange_scholars
            .alight(
                Locale::from_algebraic(
                    "e1".to_owned()));
        world.orange_princesses = world.orange_princesses
            .alight(
                Locale::from_algebraic("c3".to_owned()));
        world.blue_princesses = world.blue_princesses
            .alight(Locale::from_algebraic("g3".to_owned()));
        let premonitions = world.scholar_lookahead(Team::Orange, false);
        let expected = vec!["d2", "f2", "g3"]
                           .iter()
                           .map(|a| Locale::from_algebraic((*a).to_owned()))
                           .collect::<Vec<_>>();
        let actual = premonitions.iter()
                                 .map(|p| p.tree.orange_scholars.to_locales()[0])
                                 .collect::<Vec<_>>();
        assert_eq!(expected, actual);
    }

    #[test]
    fn concerning_occupying_agents() {
        let state = WorldState::new();
        let b8 = Locale { rank: 7, file: 1 };
        assert_eq!(Agent { team: Team::Blue,
                           job_description: JobDescription::Pony },
                   state.occupying_agent(b8).unwrap());
        let c4 = Locale { rank: 3, file: 2 };
        assert_eq!(None, state.occupying_agent(c4));
    }

    #[test]
    fn concerning_taking_turns() {
        let state1 = WorldState::new();
        let state2 = state1.lookahead()[0].tree;
        let state3 = state2.lookahead()[0].tree;
        assert_eq!(state1.to_move, Team::Orange);
        assert_eq!(state2.to_move, Team::Blue);
        assert_eq!(state3.to_move, Team::Orange);
    }

    #[test]
    fn concerning_peaceful_patch_application() {
        let state = WorldState::new();
        let e2 = Locale { rank: 4, file: 1 };
        let e4 = Locale { rank: 4, file: 3 };
        let patch = Patch {
            star: Agent {
                team: Team::Orange,
                job_description: JobDescription::Servant,
            },
            whence: e2,
            whither: e4,
            hospitalization: None,
        };
        let new_state = state.apply(patch).tree;
        assert_eq!(Agent { team: Team::Orange,
                           job_description: JobDescription::Servant },
                   new_state.occupying_agent(e4).unwrap());
        assert_eq!(None, new_state.occupying_agent(e2));
    }

    // #[ignore] // let's see if this has any hope before fixing complicated tests
    // #[test]
    // fn concerning_stunning_in_natural_setting() {
    //     let state = WorldState::new();
    //     let orange_servant_agent = Agent {
    //         team: Team::Orange,
    //         job_description: JobDescription::Servant,
    //     };
    //     let blue_servant_agent = Agent {
    //         team: Team::Blue,
    //         job_description: JobDescription::Servant,
    //     };
    //     let orange_begins = Patch {
    //         star: orange_servant_agent,
    //         whence: Locale::from_algebraic("e2".to_owned()),
    //         whither: Locale::from_algebraic("e4".to_owned()),
    //         hospitalization: None,
    //     };
    //     let blue_replies = Patch {
    //         star: blue_servant_agent,
    //         whence: Locale::from_algebraic("d7".to_owned()),
    //         whither: Locale::from_algebraic("d5".to_owned()),
    //         hospitalization: None,
    //     };
    //     let orange_counterreplies = Patch {
    //         star: orange_servant_agent,
    //         whence: Locale::from_algebraic("e4".to_owned()),
    //         whither: Locale::from_algebraic("d5".to_owned()),
    //         hospitalization: None,
    //     };

    //     let first_commit = state.apply(orange_begins);
    //     assert_eq!(None, first_commit.patch.hospitalization);
    //     let second_commit = first_commit.tree.apply(blue_replies);
    //     assert_eq!(None, second_commit.patch.hospitalization);

    //     let precrucial_state = second_commit.tree;
    //     let available_stunnings = precrucial_state.servant_lookahead(
    //         Team::Orange, false)
    //         .into_iter()
    //         .filter(|p| p.hospitalization.is_some())
    //         .collect::<Vec<_>>();
    //     assert_eq!(1, available_stunnings.len());
    //     assert_eq!(
    //         blue_servant_agent,
    //         available_stunnings[0].hospitalization.unwrap()
    //     );
    //     assert_eq!(
    //         Locale::from_algebraic("d5".to_owned()),
    //         available_stunnings[0].patch.whither
    //     );

    //     let crucial_commit = precrucial_state.apply(orange_counterreplies);
    //     let new_state = crucial_commit.tree;
    //     assert_eq!(Agent { team: Team::Orange,
    //                        job_description: JobDescription::Servant },
    //                new_state.occupying_agent(
    //                    Locale::from_algebraic("d5".to_owned())).unwrap());
    //     let stunned = crucial_commit.hospitalization.unwrap();
    //     assert_eq!(Agent { team: Team::Blue,
    //                        job_description: JobDescription::Servant },
    //                stunned);
    // }

    fn prelude_to_the_death_of_a_fool() -> WorldState {
        // https://en.wikipedia.org/wiki/Fool%27s_mate
        let mut world = WorldState::new();
        let fools_patchset = vec![
            Patch { star: Agent { team: Team::Orange,
                                  job_description: JobDescription::Servant },
                    whence: Locale::from_algebraic("f2".to_owned()),
                    whither: Locale::from_algebraic("f3".to_owned()),
                    hospitalization: None, },
            Patch { star: Agent { team: Team::Blue,
                                  job_description: JobDescription::Servant },
                    whence: Locale::from_algebraic("e7".to_owned()),
                    whither: Locale::from_algebraic("e5".to_owned()),
                    hospitalization: None, },
            Patch { star: Agent { team: Team::Orange,
                                  job_description: JobDescription::Servant },
                    whence: Locale::from_algebraic("g2".to_owned()),
                    whither: Locale::from_algebraic("g4".to_owned()),
                    hospitalization: None, },
        ];
        for patch in fools_patchset.into_iter() {
            world = world.careful_apply(patch).unwrap().tree;
        }
        world
    }

    fn death_of_a_fool() -> WorldState {
        let prelude = prelude_to_the_death_of_a_fool();
        prelude.apply(
            Patch {
                star: Agent {
                    team: Team::Blue,
                    job_description: JobDescription::Princess,
                },
                whence: Locale::from_algebraic("d8".to_owned()),
                whither: Locale::from_algebraic("h4".to_owned()),
                hospitalization: None,
            }).tree
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

    // #[ignore]  // again, see if sane before fix all tests
    // #[test]
    // fn concerning_preservation_and_reconstruction_of_historical_worlds() {
    //     // en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation#Examples
    //     let eden = WorldState::new();
    //     let book_of_eden = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq".to_owned();
    //     assert_eq!(book_of_eden, eden.preserve());
    //     assert_eq!(eden, WorldState::reconstruct(book_of_eden));

    //     let patchset = vec![
    //         Patch { star: Agent { team: Team::Orange,
    //                               job_description: JobDescription::Servant },
    //                 whence: Locale::from_algebraic("e2".to_owned()),
    //                 whither: Locale::from_algebraic("e4".to_owned()) },
    //         Patch { star: Agent { team: Team::Blue,
    //                               job_description: JobDescription::Servant },
    //                 whence: Locale::from_algebraic("c7".to_owned()),
    //                 whither: Locale::from_algebraic("c5".to_owned()) },
    //         Patch { star: Agent { team: Team::Orange,
    //                               job_description: JobDescription::Pony },
    //                 whence: Locale::from_algebraic("g1".to_owned()),
    //                 whither: Locale::from_algebraic("f3".to_owned()) },
    //     ];

    //     let book_of_patches = vec![
    //         "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq" // e3 0 1
    //             .to_owned(),
    //         "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq" // c6 0 2
    //             .to_owned(),
    //         "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq" // - 1 2
    //             .to_owned(),
    //     ];

    //     let mut world = eden;
    //     for (patch, book) in patchset.into_iter().zip(
    //         book_of_patches.into_iter()) {
    //         world = world.careful_apply(patch).unwrap().tree;
    //         assert_eq!(book, world.preserve());
    //         assert_eq!(WorldState::reconstruct(book), world);
    //     }
    // }

}
