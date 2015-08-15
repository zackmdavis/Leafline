extern crate ansi_term;

use ansi_term::Colour as Color;  // this is America


#[derive(Eq,PartialEq,Debug,Copy,Clone)]
struct Locale {
    rank: u8,
    file: u8
}

impl Locale {
    pub fn bit_index(&self) -> u32 {
        (8u32 * self.rank as u32) + self.file as u32
    }

    pub fn pinpoint(&self) -> Bitboard {
        Bitboard(2u64.pow(self.bit_index()))
    }

    pub fn is_legal(&self) -> bool {
        self.rank >= 0 && self.rank < 8 && self.file >= 0 && self.file < 8
    }

    pub fn displace(&self, offset: (i8, i8)) -> Option<Self> {
        let (rank_offset, file_offset) = offset;
        let potential_locale = Locale {
            // why is this legal? What happens when you coerce a
            // negative number to unsigned?
            rank: (self.rank as i8 + rank_offset) as u8,
            file: (self.file as i8 + file_offset) as u8
        };
        if potential_locale.is_legal() {
            Some(potential_locale)
        } else {
            None
        }
    }
}



#[derive(Eq,PartialEq,Debug,Copy,Clone)]
struct Bitboard(u64);

impl Bitboard {
    pub fn new() -> Bitboard {
        Bitboard(0)
    }

    pub fn init(starters: &Vec<Locale>) -> Bitboard {
        let mut board = Bitboard::new();
        for &starter in starters.iter() {
            board = board.alight(starter);
        }
        board
    }

    pub fn union(&self, other: Bitboard) -> Bitboard {
        let Bitboard(our_bits) = *self;
        let Bitboard(their_bits) = other;
        Bitboard(our_bits | their_bits)
    }

    pub fn intersection(&self, other: Bitboard) -> Bitboard {
        let Bitboard(our_bits) = *self;
        let Bitboard(their_bits) = other;
        Bitboard(our_bits & their_bits)
    }

    pub fn invert(&self) -> Bitboard {
        let Bitboard(our_bits) = *self;
        Bitboard(!our_bits)
    }

    pub fn alight(&self, station: Locale) -> Bitboard {
        // classic `|= (1 << n)` would probably be more efficient, huh &c.
        self.union(station.pinpoint())
    }

    pub fn quench(&self, station: Locale) -> Bitboard {
        self.intersection(station.pinpoint().invert())
    }

    pub fn query(&self, station: Locale) -> bool {
        let Bitboard(our_bits) = *self;
        let Bitboard(beacon_bits) = station.pinpoint();
        (our_bits & beacon_bits) != 0
    }

    pub fn to_locales(&self) -> Vec<Locale> {
        let mut locales = Vec::new();
        for rank in 0..8 {
            for file in 0..8 {
                let potential_locale = Locale { rank: rank, file: file };
                if self.query(potential_locale) {
                    locales.push(potential_locale);
                }
            }
        }
        locales
    }

    pub fn display(&self) {
        let Bitboard(debug) = *self;
        println!("{}", debug);
        for rank in 0..8 {
            for file in 0..8 {
                if self.query(Locale { rank: rank, file: file }) {
                    print!("• ");
                } else {
                    print!("_ ");
                }
            }
            println!("");
        }
    }
}


#[derive(Eq,PartialEq,Debug,Copy,Clone)]
enum Team { Orange, Blue }

impl Team {
    fn opponent(&self) -> Self {
        match self {
            &Team::Orange => Team::Blue,
            &Team::Blue => Team::Orange
        }
    }
}

#[derive(Eq,PartialEq,Debug,Copy,Clone)]
enum JobDescription {
    Servant,  // ♂
    Pony,  // ♀
    Scholar,  // ♀
    Cop,  // ♂
    Princess,  // ♀
    Figurehead  // ♂
}

#[derive(Eq,PartialEq,Debug,Copy,Clone)]
struct Agent {
    team: Team,
    job_description: JobDescription
}

impl Agent {
    // I wanted to call it `dramatis_personæ`, but "non-ascii idents
    // are not fully supported" 🙀
    pub fn dramatis_personae(team: Team) -> Vec<Agent> {
        vec![Agent{ team: team,
                    job_description: JobDescription::Servant },
             Agent{ team: team,
                    job_description: JobDescription::Pony },
             Agent{ team: team,
                    job_description: JobDescription::Scholar },
             Agent{ team: team,
                    job_description: JobDescription::Cop },
             Agent{ team: team,
                    job_description: JobDescription::Princess },
             Agent{ team: team,
                    job_description: JobDescription::Figurehead }]
    }

    pub fn render_caricature(&self) {
        let caricature = match self {
            &Agent { team: Team::Orange, .. } => {
                match self.job_description {
                    JobDescription::Servant => Color::Yellow.paint("♙"),
                    JobDescription::Pony => Color::Yellow.paint("♘"),
                    JobDescription::Scholar => Color::Yellow.paint("♗"),
                    JobDescription::Cop => Color::Yellow.paint("♖"),
                    JobDescription::Princess => Color::Yellow.paint("♕"),
                    JobDescription::Figurehead => Color::Yellow.paint("♔"),
                }
            },
            &Agent { team: Team::Blue, .. } => {
                match self.job_description {
                    JobDescription::Servant => Color::Cyan.paint("♟"),
                    JobDescription::Pony => Color::Cyan.paint("♞"),
                    JobDescription::Scholar => Color::Cyan.paint("♝"),
                    JobDescription::Cop => Color::Cyan.paint("♜"),
                    JobDescription::Princess => Color::Cyan.paint("♛"),
                    JobDescription::Figurehead => Color::Cyan.paint("♚"),
                }
            }
        };
        print!("{}", caricature);
    }

}


#[derive(Eq,PartialEq,Debug,Copy,Clone)]
struct GameState {
    to_move: Team,

    orange_servants: Bitboard,
    orange_ponies: Bitboard,
    orange_scholars: Bitboard,
    orange_cops: Bitboard,
    orange_princesses: Bitboard,
    orange_figurehead: Bitboard,

    blue_servants: Bitboard,
    blue_ponies: Bitboard,
    blue_scholars: Bitboard,
    blue_cops: Bitboard,
    blue_princesses: Bitboard,
    blue_figurehead: Bitboard,
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

            orange_servants: Bitboard::init(&orange_servant_locales),
            orange_ponies: Bitboard::init(
                &vec![Locale { rank: 0, file: 1 },
                      Locale { rank: 0, file: 6 }]
            ),
            orange_scholars: Bitboard::init(
                &vec![Locale { rank: 0, file: 2 },
                      Locale { rank: 0, file: 5 }]
            ),
            orange_cops: Bitboard::init(
                &vec![Locale { rank: 0, file: 0 },
                      Locale { rank: 0, file: 7 }]
            ),
            orange_princesses: Bitboard::init(
                &vec![Locale { rank: 0, file: 3 }]),
            orange_figurehead: Bitboard::init(
                &vec![Locale { rank: 0, file: 4 }]),
            blue_servants: Bitboard::init(&blue_servant_locales),
            blue_ponies: Bitboard::init(
                &vec![Locale { rank: 7, file: 1 },
                      Locale { rank: 7, file: 6 }]
            ),
            blue_scholars: Bitboard::init(
                &vec![Locale { rank: 7, file: 2 },
                      Locale { rank: 7, file: 5 }]
            ),
            blue_cops: Bitboard::init(
                &vec![Locale { rank: 7, file: 0 },
                      Locale { rank: 7, file: 7 }]
            ),
            blue_princesses: Bitboard::init(
                &vec![Locale { rank: 7, file: 3 }]),
            blue_figurehead: Bitboard::init(
                &vec![Locale { rank: 7, file: 4 }]),
        }
    }

    pub fn agent_to_bitboard_ref(&self, agent: Agent) -> &Bitboard {
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

    pub fn occupied_by(&self, team: Team) -> Bitboard {
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

    pub fn occupied(&self) -> Bitboard {
        self.occupied_by(Team::Orange).union(self.occupied_by(Team::Blue))
    }

    pub fn unoccupied(&self) -> Bitboard {
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
                standard_offset = (0, 1);
                boost_offset = (0, 2);
                stun_offsets = [(1, -1), (1, 1)];
            },
            Team::Blue => {
                initial_rank = 6;
                standard_offset = (0, -1);
                boost_offset = (0, -2);
                stun_offsets = [(-1, -1), (-1, 1)];
            }
        }
        let mut possibilities = Vec::new();
        let positional_chart: &Bitboard = self.agent_to_bitboard_ref(
            Agent { team: team, job_description: JobDescription::Servant });
        for start_locale in positional_chart.to_locales().iter() {
            // TODO can move one square
            if start_locale.rank == initial_rank {
                // TODO can move with the two-square "boost offset"
            }
            // TODO can move diagonally
        }
        possibilities
    }

    pub fn lookahead(&self) -> Vec<Self> {
        let possibilities = Vec::new();
        let moving_team = self.to_move;
        for agent_class in Agent::dramatis_personae(moving_team) {
            let positional_chart: &Bitboard = self.agent_to_bitboard_ref(
                agent_class);
            // for each agent class, compute possible moves ... somehow
        }
        // TODO work in progress
        possibilities
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
                                if self.agent_to_bitboard_ref(
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
    arena.occupied().display();
    arena.occupied().invert().display();
}


#[cfg(test)]
mod test {
    use super::{Locale, Bitboard, Team, JobDescription, Agent, GameState};

    #[test]
    fn test_locale() {
        let a1 = Locale { rank: 0, file: 0 };
        assert_eq!(Bitboard(1u64), a1.pinpoint());

        let c4 = Locale { rank: 2, file: 3 };
        assert_eq!(Bitboard(524288u64), c4.pinpoint());
    }

    #[test]
    fn test_displace_locale() {
        assert_eq!(
            Some(Locale { rank: 3, file: 5 }),
            Locale { rank: 4, file: 4 }.displace((-1, 1))
        );
        assert_eq!(
            None,
            Locale { rank: 0, file: 0 }.displace((-1, 1))
        )
    }

    #[test]
    fn test_empty_board_is_empty() {
        let empty_board = Bitboard::new();
        for rank in 0..8 {
            for file in 0..8 {
                assert!(!empty_board.query(Locale { rank: rank, file: file }));
            }
        }
    }

    #[test]
    fn test_inverted_empty_board_is_full() {
        let full_board = Bitboard::new().invert();
        for rank in 0..8 {
            for file in 0..8 {
                assert!(full_board.query(Locale { rank: rank, file: file }));
            }
        }
    }

    #[test]
    fn test_alight_and_quench() {
        let mut stage = Bitboard(0);
        let b5 = Locale { rank: 1, file: 4 };
        stage = stage.alight(b5);
        assert!(stage.query(b5));
        stage = stage.quench(b5);
        assert!(!stage.query(b5));
    }

    #[test]
    fn test_init_bitboard() {
        let starters = vec![Locale{ rank: 1, file: 2 },
                            Locale{ rank: 3, file: 4 },
                            Locale{ rank: 5, file: 6 }];
        let stage = Bitboard::init(&starters);
        for &starter in starters.iter() {
            assert!(stage.query(starter));
        }
    }

    #[test]
    fn test_agent_to_bitboard_ref_on_new_gamestate() {
        let state = GameState::new();
        let agent = Agent { team: Team::Blue,
                            job_description: JobDescription::Princess };
        let blue_princess_realm = state.agent_to_bitboard_ref(agent);
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

}
