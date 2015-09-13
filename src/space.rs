#[derive(Eq,PartialEq,Debug,Copy,Clone,Hash,RustcEncodable,RustcDecodable)]
pub struct Locale {
    pub rank: u8,
    pub file: u8,
}

static INDEX_TO_FILE_NAME: [char; 8] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'
];

impl Locale {

    pub fn to_algebraic(&self) -> String {
        format!("{}{}", INDEX_TO_FILE_NAME[self.file as usize], self.rank + 1)
    }

    // XXX: this should probably take a &str instead of String,
    // because the argument is typically going to be a string literal
    // rather than a value from somewhere else
    #[allow(dead_code)]
    pub fn from_algebraic(notation: String) -> Self {
        let mut notation_pieces = notation.chars();
        let file_note = notation_pieces.next().unwrap();
        let rank_note = notation_pieces.next().unwrap();
        Locale {
            rank: (rank_note as u8) - 49, // 49 == '1'
            file: (file_note as u8) - 97, // 97 == 'a'
        }
    }

    pub fn pindex(&self) -> u32 {
        (8u32 * self.rank as u32) + self.file as u32
    }

    pub fn pinpoint(&self) -> Pinfield {
        Pinfield(1u64 << self.pindex())
    }

    pub fn is_legal(&self) -> bool {
        self.rank < 8 && self.file < 8
    }

    pub fn displace(&self, offset: (i8, i8)) -> Option<Self> {
        let (rank_offset, file_offset) = offset;
        let potential_locale = Locale {
            // XXX: it won't happen with the arguments we expect to
            // give it in this program, but in the interests of Safety,
            // this is an overflow bug (-1i8 as u8 == 255u8)
            rank: (self.rank as i8 + rank_offset) as u8,
            file: (self.file as i8 + file_offset) as u8,
        };
        if potential_locale.is_legal() {
            Some(potential_locale)
        } else {
            None
        }
    }

    pub fn multidisplace(&self, offset: (i8, i8), factor: i8) -> Option<Self> {
        let (rank_offset, file_offset) = offset;
        let (real_rank, real_file) = (factor * rank_offset,
                                      factor * file_offset);

        let potential_locale = Locale {
            // XXX: could overflow given unrealistic arguments
            rank: (self.rank as i8 + real_rank) as u8,
            file: (self.file as i8 + real_file) as u8,
        };
        if potential_locale.is_legal() {
            Some(potential_locale)
        } else {
            None
        }
    }

}


#[derive(Eq,PartialEq,Debug,Copy,Clone,Hash)]
pub struct Pinfield(pub u64);

impl Pinfield {
    pub fn new() -> Pinfield {
        Pinfield(0)
    }

    pub fn init(starters: &Vec<Locale>) -> Pinfield {
        let mut board = Pinfield::new();
        for &starter in starters.iter() {
            board = board.alight(starter);
        }
        board
    }

    pub fn union(&self, other: Pinfield) -> Pinfield {
        let Pinfield(our_bits) = *self;
        let Pinfield(their_bits) = other;
        Pinfield(our_bits | their_bits)
    }

    pub fn intersection(&self, other: Pinfield) -> Pinfield {
        let Pinfield(our_bits) = *self;
        let Pinfield(their_bits) = other;
        Pinfield(our_bits & their_bits)
    }

    pub fn invert(&self) -> Pinfield {
        let Pinfield(our_bits) = *self;
        Pinfield(!our_bits)
    }

    pub fn difference(&self, other: Pinfield) -> Pinfield {
        let Pinfield(our_bits) = *self;
        let Pinfield(their_bits) = other;
        Pinfield(our_bits & !their_bits)
    }

    pub fn alight(&self, station: Locale) -> Pinfield {
        // classic `|= (1 << n)` would probably be more efficient, huh &c.
        self.union(station.pinpoint())
    }

    pub fn quench(&self, station: Locale) -> Pinfield {
        self.intersection(station.pinpoint().invert())
    }

    pub fn transit(&self, departure: Locale, destination: Locale) -> Pinfield {
        self.quench(departure).alight(destination)
    }

    pub fn query(&self, station: Locale) -> bool {
        let Pinfield(our_bits) = *self;
        let Pinfield(beacon_bits) = station.pinpoint();
        (our_bits & beacon_bits) != 0
    }

    pub fn to_locales(&self) -> Vec<Locale> {
        let mut locales = Vec::new();
        let Pinfield(bits) = *self;
        let mut bitfield = 1u64;
        for rank in 0..8 {
            for file in 0..8 {
                if bitfield & bits != 0 {
                    let locale = Locale { rank: rank, file: file };
                    locales.push(locale);
                }
                bitfield <<= 1;
            }
        }
        locales
    }

    pub fn pincount(&self) -> u8 {
        let Pinfield(bits) = *self;
        bits.count_ones() as u8
    }

    // TODO: convert to Display::fmt
    #[allow(dead_code)]
    pub fn display(&self) {
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


#[cfg(test)]
mod tests {
    extern crate test;
    use self::test::Bencher;
    use super::{Locale, Pinfield};

    static ALGEBRAICS: [&'static str; 64] = [
        "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1",
        "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
        "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3",
        "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
        "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5",
        "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
        "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7",
        "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8"
    ];

    #[ignore]
    #[bench]
    fn benchmark_to_locales(b: &mut Bencher) {
        let mut stage = Pinfield(0);
        for r in 0..8 {
            stage = stage.alight(Locale { rank: 1, file: r });
            stage = stage.alight(Locale { rank: 7, file: r });
        }

        b.iter(|| stage.to_locales());
    }

    #[test]
    fn concerning_converting_to_algebraic() {
        let actual = iproduct!(0..8, 0..8)
                         .map(|t| Locale { rank: t.0, file: t.1 })
                         .map(|l| l.to_algebraic());
        for (expectation, actuality) in ALGEBRAICS.iter().zip(actual) {
            // TODO: it's more elegant if the conversion happens in
            // the iterator rather than the body of this
            // assertion-iteration
            assert_eq!(expectation.to_owned(), actuality);
        }
    }

    #[test]
    fn concerning_converting_from_algebraic() {
        let expected = iproduct!(0..8, 0..8).map(
            |t| Locale { rank: t.0, file: t.1 });
        for (expectation, actuality) in expected.zip(ALGEBRAICS.iter()) {
            assert_eq!(
                expectation,
                // TODO: again, conversion in iterator
                Locale::from_algebraic((*actuality).to_owned())
            );
        }
    }

    #[test]
    fn test_locale() {
        let a1 = Locale { rank: 0, file: 0 };
        assert_eq!(Pinfield(1u64), a1.pinpoint());

        let c4 = Locale { rank: 2, file: 3 };
        assert_eq!(Pinfield(524288u64), c4.pinpoint());
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
    fn concerning_multidisplacement() {
        for i in 0..8 {
            assert_eq!(
                Some(Locale { rank: i as u8, file: i as u8 }),
                Locale{ rank: 0, file: 0 }.multidisplace((1, 1), i)
            )
        }
        assert_eq!(None, Locale{ rank: 0, file: 0 }.multidisplace((1, 1), 8));
    }


    #[test]
    fn test_alight_and_quench() {
        let mut stage = Pinfield(0);
        let b5 = Locale { rank: 1, file: 4 };
        stage = stage.alight(b5);
        assert!(stage.query(b5));
        stage = stage.quench(b5);
        assert!(!stage.query(b5));
    }

    #[test]
    fn test_init_pinfield() {
        let starters = vec![Locale{ rank: 1, file: 2 },
                            Locale{ rank: 3, file: 4 },
                            Locale{ rank: 5, file: 6 }];
        let stage = Pinfield::init(&starters);
        for &starter in &starters {
            assert!(stage.query(starter));
        }
    }

    #[test]
    fn concerning_pincount() {
        let starters = vec![Locale{ rank: 1, file: 2 },
                            Locale{ rank: 3, file: 4 },
                            Locale{ rank: 5, file: 6 }];
        let stage = Pinfield::init(&starters);
        assert_eq!(3, stage.pincount());
    }
}
