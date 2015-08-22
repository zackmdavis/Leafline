extern crate ansi_term;

mod space;
mod identity;
mod motion;

use space::{Locale, Pinfield};
use identity::{Team, JobDescription, Agent};
use motion::{PONY_MOVEMENT_TABLE, FIGUREHEAD_MOVEMENT_TABLE};


#[derive(Eq,PartialEq,Debug,Copy,Clone)]
struct GameState {
    to_move: Team,

    orange_servants: Pinfield,
    orange_ponies: Pinfield,
    orange_scholars: Pinfield,
    orange_cops: Pinfield,
    orange_princesses: Pinfield,
    orange_figurehead: Pinfield,

    blue_servants: Pinfield,
    blue_ponies: Pinfield,
    blue_scholars: Pinfield,
    blue_cops: Pinfield,
    blue_princesses: Pinfield,
    blue_figurehead: Pinfield,
}

impl GameState {
    pub fn new() -> GameState {
        let mut orange_servant_locales = Vec::new();
        let mut blue_servant_locales = Vec::new();
        for f in 0..8 {
            orange_servant_locales.push(Locale { rank: 1, file: f });
            blue_servant_locales.push(Locale { rank: 6, file: f });
        }
        GameState {
            to_move: Team::Orange,

            orange_servants: Pinfield::init(&orange_servant_locales),
            orange_ponies: Pinfield::init(
                &vec![Locale { rank: 0, file: 1 },
                      Locale { rank: 0, file: 6 }]
            ),
            orange_scholars: Pinfield::init(
                &vec![Locale { rank: 0, file: 2 },
                      Locale { rank: 0, file: 5 }]
            ),
            orange_cops: Pinfield::init(
                &vec![Locale { rank: 0, file: 0 },
                      Locale { rank: 0, file: 7 }]
            ),
            orange_princesses: Pinfield::init(
                &vec![Locale { rank: 0, file: 3 }]),
            orange_figurehead: Pinfield::init(
                &vec![Locale { rank: 0, file: 4 }]),
            blue_servants: Pinfield::init(&blue_servant_locales),
            blue_ponies: Pinfield::init(
                &vec![Locale { rank: 7, file: 1 },
                      Locale { rank: 7, file: 6 }]
            ),
            blue_scholars: Pinfield::init(
                &vec![Locale { rank: 7, file: 2 },
                      Locale { rank: 7, file: 5 }]
            ),
            blue_cops: Pinfield::init(
                &vec![Locale { rank: 7, file: 0 },
                      Locale { rank: 7, file: 7 }]
            ),
            blue_princesses: Pinfield::init(
                &vec![Locale { rank: 7, file: 3 }]),
            blue_figurehead: Pinfield::init(
                &vec![Locale { rank: 7, file: 4 }]),
        }
    }

    pub fn agent_to_pinfield_ref(&self, agent: Agent) -> &Pinfield {
        match agent {
            Agent{ team: Team::Orange,
                   job_description: JobDescription::Servant } =>
                &self.orange_servants,
            Agent{ team: Team::Orange,
                   job_description: JobDescription::Pony } =>
                &self.orange_ponies,
            Agent{ team: Team::Orange,
                   job_description: JobDescription::Scholar } =>
                &self.orange_scholars,
            Agent{ team: Team::Orange,
                   job_description: JobDescription::Cop } =>
                &self.orange_cops,
            Agent{ team: Team::Orange,
                   job_description: JobDescription::Princess } =>
                &self.orange_princesses,
            Agent{ team: Team::Orange,
                   job_description: JobDescription::Figurehead } =>
                &self.orange_figurehead,
            Agent{ team: Team::Blue,
                   job_description: JobDescription::Servant } =>
                &self.blue_servants,
            Agent{ team: Team::Blue,
                   job_description: JobDescription::Pony } =>
                &self.blue_ponies,
            Agent{ team: Team::Blue,
                   job_description: JobDescription::Scholar } =>
                &self.blue_scholars,
            Agent{ team: Team::Blue,
                   job_description: JobDescription::Cop } =>
                &self.blue_cops,
            Agent{ team: Team::Blue,
                   job_description: JobDescription::Princess } =>
                &self.blue_princesses,
            Agent{ team: Team::Blue,
                   job_description: JobDescription::Figurehead } =>
                &self.blue_figurehead,
        }
    }

    // XXX this code-duplication is hideous, but what can you do in
    // this language? My problem is exactly that I don't know
    pub fn agent_to_pinfield_mutref(&mut self, agent: Agent) -> &mut Pinfield {
        match agent {
            Agent{ team: Team::Orange,
                   job_description: JobDescription::Servant } =>
                &mut self.orange_servants,
            Agent{ team: Team::Orange,
                   job_description: JobDescription::Pony } =>
                &mut self.orange_ponies,
            Agent{ team: Team::Orange,
                   job_description: JobDescription::Scholar } =>
                &mut self.orange_scholars,
            Agent{ team: Team::Orange,
                   job_description: JobDescription::Cop } =>
                &mut self.orange_cops,
            Agent{ team: Team::Orange,
                   job_description: JobDescription::Princess } =>
                &mut self.orange_princesses,
            Agent{ team: Team::Orange,
                   job_description: JobDescription::Figurehead } =>
                &mut self.orange_figurehead,
            Agent{ team: Team::Blue,
                   job_description: JobDescription::Servant } =>
                &mut self.blue_servants,
            Agent{ team: Team::Blue,
                   job_description: JobDescription::Pony } =>
                &mut self.blue_ponies,
            Agent{ team: Team::Blue,
                   job_description: JobDescription::Scholar } =>
                &mut self.blue_scholars,
            Agent{ team: Team::Blue,
                   job_description: JobDescription::Cop } =>
                &mut self.blue_cops,
            Agent{ team: Team::Blue,
                   job_description: JobDescription::Princess } =>
                &mut self.blue_princesses,
            Agent{ team: Team::Blue,
                   job_description: JobDescription::Figurehead } =>
                &mut self.blue_figurehead,
        }
    }

    pub fn except_replaced_subboard(&self, for_whom: Agent,
                                    subboard: Pinfield) -> Self {
        let mut resultant_state = self.clone();
        resultant_state.agent_to_pinfield_mutref(for_whom).0 = subboard.0;
        resultant_state
    }



    pub fn occupied_by(&self, team: Team) -> Pinfield {
        match team {
            Team::Orange => self.orange_servants.union(
                self.orange_ponies).union(
                    self.orange_scholars).union(self.orange_cops).union(
                        self.orange_princesses).union(self.orange_figurehead),
            Team::Blue => self.blue_servants.union(self.blue_ponies).union(
                    self.blue_scholars).union(self.blue_cops).union(
                            self.blue_princesses).union(self.blue_figurehead)
        }
    }

    pub fn occupied(&self) -> Pinfield {
        self.occupied_by(Team::Orange).union(self.occupied_by(Team::Blue))
    }

    pub fn unoccupied(&self) -> Pinfield {
        self.occupied().invert()
    }

    fn servant_lookahead(&self, team: Team) -> Vec<GameState> {
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
            },
            Team::Blue => {
                initial_rank = 6;
                standard_offset = (-1, 0);
                boost_offset = (-2, 0);
                stun_offsets = [(-1, -1), (-1, 1)];
            }
        }
        let mut premonitions = Vec::new();
        let servant_agent = Agent {
            team: team, job_description: JobDescription::Servant };
        let positional_chart: &Pinfield = self.agent_to_pinfield_ref(
            servant_agent);
        for start_locale in positional_chart.to_locales().iter() {
            // can move one locale if not blocked
            let std_destination_maybe = start_locale.displace(standard_offset);
            if let Some(destination_locale) = std_destination_maybe {
                if self.unoccupied().query(destination_locale) {
                    let mut premonition = self.clone();
                    premonition = premonition.except_replaced_subboard(
                        servant_agent, positional_chart.transit(
                            *start_locale, destination_locale));
                    premonitions.push(premonition);
                }
            }
            // can move two locales if he hasn't previously moved
            if start_locale.rank == initial_rank {
                // safe to unwrap because we know that we're at the
                // initial rank
                let boost_destination = start_locale.displace(
                    boost_offset).unwrap();
                let standard_destination = start_locale.displace(
                    standard_offset).unwrap();
                if (self.unoccupied().query(boost_destination) &&
                    self.unoccupied().query(standard_destination)) {
                    let mut premonition = self.clone();
                    premonition.except_replaced_subboard(
                        servant_agent, positional_chart.transit(
                            *start_locale, boost_destination));
                    premonitions.push(premonition);
                }
            }
            // TODO can stun diagonally
        }
        premonitions
    }

    fn pony_lookahead(&self, team: Team) -> Vec<GameState> {
        let mut premonitions = Vec::new();
        let pony_agent = Agent {
            team: team, job_description: JobDescription::Pony };
        let positional_chart: &Pinfield = self.agent_to_pinfield_ref(
            pony_agent);
        for start_locale in positional_chart.to_locales().iter() {
            let destinations = self.occupied_by(team).invert().intersection(
                Pinfield(PONY_MOVEMENT_TABLE[
                    start_locale.bit_index() as usize])).to_locales();
            for destination in destinations.iter() {
                let mut premonition = self.clone();
                premonition = premonition.except_replaced_subboard(
                    pony_agent, positional_chart.transit(
                        *start_locale, *destination));
                // TODO put any stunned opposing figuring into hospital
                premonitions.push(premonition);
            }
        }
        premonitions
    }

    pub fn lookahead(&self) -> Vec<Self> {
        let premonitions = Vec::new();
        let moving_team = self.to_move;


        // TODO work in progress
        premonitions
    }

    pub fn display(&self) {
        println!("  a b c d e f g h");
        for rank in 0..8 {
            print!("{} ", rank+1);
            for file in 0..8 {
                let locale = Locale { rank: rank, file: file };
                if self.occupied().invert().query(locale) {
                    print!("_ ");
                } else {
                    for &team in [Team::Orange, Team::Blue].iter() {
                        for &figurine_class in
                            Agent::dramatis_personae(team).iter() {
                                if self.agent_to_pinfield_ref(
                                    figurine_class).query(locale) {
                                        figurine_class.render_caricature();
                                        print!(" ");
                                }
                        }
                    }
                }
            }
            println!("");
        }
    }
}


fn main() {
    let arena = GameState::new();
    arena.display();
    println!("");
}


#[cfg(test)]
mod test {
    use super::GameState;
    use space::{Locale, Pinfield};
    use identity::{Team, JobDescription, Agent};

    #[test]
    fn test_agent_to_pinfield_ref_on_new_gamestate() {
        let state = GameState::new();
        let agent = Agent { team: Team::Blue,
                            job_description: JobDescription::Princess };
        let blue_princess_realm = state.agent_to_pinfield_ref(agent);
        assert!(blue_princess_realm.query(Locale { rank: 7, file: 3 }));
    }

    #[test]
    fn test_orange_servants_to_locales_from_new_gamestate() {
        let state = GameState::new();
        let mut expected = Vec::new();
        for file in 0..8 {
            expected.push(Locale { rank: 1, file: file });
        }
        assert_eq!(expected, state.orange_servants.to_locales());
    }

    #[test]
    fn test_orange_servant_lookahead_from_original_position() {
        let state = GameState::new();
        let premonitions = state.servant_lookahead(Team::Orange);
        assert_eq!(16, premonitions.len());
        // although granted that a more thorough test would actually
        // say something about the nature of the positions, rather than
        // just how many there are
    }

    #[test]
    fn test_orange_pony_lookahead_from_original_position() {
        let state = GameState::new();
        let premonitions = state.pony_lookahead(Team::Orange);
        assert_eq!(4, premonitions.len());
        let collected = premonitions.iter().map(
            |p| p.orange_ponies.to_locales()).collect::<Vec<_>>();
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

}
