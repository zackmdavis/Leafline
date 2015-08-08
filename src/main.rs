#[derive(Eq,PartialEq,Debug,Copy,Clone)]
struct Locale {
    rank: u8,
    file: u8
}

impl Locale {
    fn bit_index(&self) -> u32 {
        (8u32 * self.rank as u32) + self.file as u32
    }

    fn pinpoint(&self) -> Bitboard {
        Bitboard(2u64.pow(self.bit_index()))
    }
}

#[derive(Eq,PartialEq,Debug)]
struct Bitboard(u64);

impl Bitboard {
    fn union(&self, other: Bitboard) -> Bitboard {
        let Bitboard(our_bits) = *self;
        let Bitboard(their_bits) = other;
        Bitboard(our_bits | their_bits)
    }

    fn intersection(&self, other: Bitboard) -> Bitboard {
        let Bitboard(our_bits) = *self;
        let Bitboard(their_bits) = other;
        Bitboard(our_bits & their_bits)
    }

    fn invert(&self) -> Bitboard {
        let Bitboard(our_bits) = *self;
        Bitboard(our_bits ^ our_bits)
    }

    fn alight(&self, station: Locale) -> Bitboard {
        // classic `|= (1 << n)` would probably be more efficient, huh &c.
        self.union(station.pinpoint())
    }

    fn quench(&self, station: Locale) -> Bitboard {
        self.intersection(station.pinpoint().invert())
    }

    fn query(&self, station: Locale) -> bool {
        let Bitboard(our_bits) = *self;
        let Bitboard(beacon_bits) = station.pinpoint();
        (our_bits & beacon_bits) != 0
    }
}

enum Teams { Blue, Orange }

struct GameState {
    blue_servants: Bitboard,
    blue_ponies: Bitboard,
    blue_scholars: Bitboard,
    blue_cops: Bitboard,
    blue_princesses: Bitboard,
    blue_figurehead: Bitboard,
    orange_servants: Bitboard,
    orange_ponies: Bitboard,
    orange_scholars: Bitboard,
    orange_cops: Bitboard,
    orange_princesses: Bitboard,
    orange_figurehead: Bitboard,
}


#[test]
fn test_locale() {
    let a1 = Locale { rank: 0, file: 0 };
    assert_eq!(Bitboard(1u64), a1.pinpoint());

    let c4 = Locale { rank: 2, file: 3 };
    assert_eq!(Bitboard(524288u64), c4.pinpoint());
}

#[test]
fn test_empty_board_is_empty() {
    let empty_board = Bitboard(0);
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
