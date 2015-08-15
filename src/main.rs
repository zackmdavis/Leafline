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
}

#[derive(Eq,PartialEq,Debug)]
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
        Bitboard(our_bits ^ our_bits)
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
}


#[derive(Eq,PartialEq,Debug)]
enum Team { Orange, Blue }

#[derive(Eq,PartialEq,Debug)]
enum JobDescription { Servant, Pony, Scholar, Cop, Princess, Figurehead }

#[derive(Eq,PartialEq,Debug)]
struct Agent {
    team: Team,
    job_description: JobDescription
}

#[derive(Eq,PartialEq,Debug)]
struct GameState {
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
            orange_servant_locales.push(Locale { rank: 6, file: f });
            blue_servant_locales.push(Locale { rank: 1, file: f });
        }
        GameState {
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

    pub fn agent_to_bitboard_ref(&self, agent: Agent) ->
        // I'm actually going to return a Bitboard from this function,
        // but am writing something different first as an experiment to see if
        // I understand how `match` works.
        &str {
        match agent {
            Agent{ team: Team::Orange, .. } => "orange!",
            Agent{ team: Team::Blue, .. } => "blue!"
        }
        // TOOD
    }

    pub fn display(&self) {
        for rank in 0..8 {
            for file in 0..8 {
                // TODO
            }
        }

    }

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
    fn test_empty_board_is_empty() {
        let empty_board = Bitboard::new();
        for rank in 0..8 {
            for file in 0..8 {
                assert!(!empty_board.query(Locale { rank: rank, file: file }));
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
    fn test_agent_to_bitboard_ref_on_new_stage() {
        let stage = GameState::new();
        let agent = Agent { team: Team::Blue,
                            job_description: JobDescription::Princess };
        assert_eq!("blue!", stage.agent_to_bitboard_ref(agent));
    }

}
