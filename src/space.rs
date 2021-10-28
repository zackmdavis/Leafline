#[derive(Eq,PartialEq,Debug,Copy,Clone,Hash,RustcEncodable,RustcDecodable)]
pub struct Locale {
    rank_and_file: u8,
}

#[derive(Eq,PartialEq,Debug,Copy,Clone,Hash,RustcEncodable,RustcDecodable)]
pub struct RelaxedLocale {
    rank: u8,
    file: u8,
}

// A less-compressed representation of a Locale, for compatibility with the
// legacy web client
impl From<Locale> for RelaxedLocale {
    fn from(locale: Locale) -> Self {
        Self { rank: locale.rank(), file: locale.file() }
    }
}

#[allow(clippy::identity_op)]
pub const ORANGE_FIGUREHEAD_START: Locale = Locale { rank_and_file: (0 << 4) | 4 };
pub const BLUE_FIGUREHEAD_START: Locale = Locale { rank_and_file: (7 << 4) | 4 };

static INDEX_TO_FILE_NAME: [char; 8] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'
];

lazy_static! {
    static ref LOCALE_STASH: [Locale; 64] = {
        let mut m: [Locale; 64] = [Locale {rank_and_file: 0}; 64];
        for rank in 0..8 {
            for file in 0..8 {
                let idx = (rank*8 + file) as usize;

                let rank_and_file = (rank << 4) | file;
                m[idx] = Locale { rank_and_file }
            }
        }
        m
    };
}


impl Locale {
    pub fn new(rank: u8, file: u8) -> Self {
        let idx = (rank*8 + file) as usize;
        LOCALE_STASH[idx]
    }

    pub fn to_algebraic(&self) -> String {
        format!("{}{}", INDEX_TO_FILE_NAME[self.file() as usize], self.rank() + 1)
    }

    pub fn from_algebraic(notation: &str) -> Self {
        let mut notation_pieces = notation.chars();
        let file_note = notation_pieces.next()
            .expect("expected a first character");
        let rank_note = notation_pieces.next()
            .expect("expected a second character");
        Locale::new(
            (rank_note as u8) - 49, // 49 == '1'
            (file_note as u8) - 97, // 97 == 'a'
        )
    }

    pub fn pindex(&self) -> u32 {
        (8u32 * u32::from(self.rank())) + u32::from(self.file())
    }

    pub fn pinpoint(&self) -> Pinfield {
        Pinfield(1u64 << self.pindex())
    }

    pub fn is_legal(&self) -> bool {
        self.rank() < 8 && self.file() < 8
    }

    fn build_possibly_illegal(rank: u8, file: u8) -> Self {
        let rank_and_file = (rank << 4) | file;
        Locale { rank_and_file }
    }

    pub fn displace(&self, offset: (i8, i8)) -> Option<Self> {
        let (rank_offset, file_offset) = offset;
        // note: when constructing possibly-illegal Locales, do not use ::new
        let potential_locale = Locale::build_possibly_illegal(
            // XXX: it won't happen with the arguments we expect to
            // give it in this program, but in the interests of Safety,
            // this is an overflow bug (-1i8 as u8 == 255u8)
            (self.rank() as i8 + rank_offset) as u8,
            (self.file() as i8 + file_offset) as u8,
        );
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

        // note: when constructing possibly-illegal Locales, do not use ::new
        let potential_locale = Locale::build_possibly_illegal(
            // XXX: could overflow given unrealistic arguments
            (self.rank() as i8 + real_rank) as u8,
            (self.file() as i8 + real_file) as u8,
        );
        if potential_locale.is_legal() {
            Some(potential_locale)
        } else {
            None
        }
    }

    pub fn rank(&self) -> u8 {
        self.rank_and_file >> 4
    }

    pub fn file(&self) -> u8 {
        self.rank_and_file & (0b1111)
    }
}


#[derive(Eq, PartialEq, Debug, Copy, Clone, Hash, Default)]
pub struct Pinfield(pub u64);

impl Pinfield {
    pub fn new() -> Self {
        Pinfield(0)
    }

    pub fn init(starters: &[Locale]) -> Pinfield {
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

    pub fn alight(&self, station: Locale) -> Pinfield {
        // classic `|= (1 << n)` would probably be more efficient, huh &c.
        self.union(station.pinpoint())
    }

    pub fn quench(&self, station: Locale) -> Pinfield {
        self.intersection(station.pinpoint().invert())
    }

    pub fn transit(&self, departure: Locale, destination: Locale) -> Pinfield {
        // empirically, pulling function calls out of this and replacing
        // them with bit manipulation directly doesnt help.
        self.quench(departure).alight(destination)
    }

    pub fn query(&self, station: Locale) -> bool {
        let Pinfield(our_bits) = *self;
        let Pinfield(beacon_bits) = station.pinpoint();
        (our_bits & beacon_bits) != 0
    }

    pub fn to_locales(&self) -> Vec<Locale> {
        let mut locales = Vec::with_capacity(8);
        let Pinfield(bits) = *self;
        let mut bitfield = 1u64;
        for rank in 0..8 {
            for file in 0..8 {
                if bitfield & bits != 0 {
                    let locale = Locale::new(rank, file);
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
                if self.query(Locale::new(rank, file)) {
                    print!("• ");
                } else {
                    print!("_ ");
                }
            }
            println!();
        }
    }
}


#[cfg(test)]
mod tests {
    extern crate test;
    extern crate rand;
    use self::test::{Bencher, black_box};
    use super::{Locale, Pinfield};
    use fnv;
    use twox_hash::XxHash;
    use std::hash::Hash;
    use std::collections::hash_map;
    use space::tests::rand::prelude::*;

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

    #[bench]
    fn benchmark_hashing_tuple_fnv(b: &mut Bencher) {
        let mut hasher = fnv::FnvHasher::default();
        let t: (u8, u8) = (1, 4);

        b.iter(|| {
            for _ in 0..1000 {
                t.hash(&mut hasher);
            }
        });
    }


    #[bench]
    fn benchmark_hashing_tuple_xx(b: &mut Bencher) {
        let mut hasher = XxHash::default();
        let t: (u8, u8) = (1, 4);

        b.iter(|| {
            for _ in 0..1000 {
                t.hash(&mut hasher);
            }
        });
    }


    #[bench]
    fn benchmark_hashing_tuple_sip(b: &mut Bencher) {
        let mut hasher = hash_map::DefaultHasher::new();
        let t: (u8, u8) = (1, 4);

        b.iter(|| {
            for _ in 0..1000 {
                t.hash(&mut hasher);
            }
        });
    }


    #[bench]
    fn benchmark_locale_lookup(b: &mut Bencher) {
        let mut args = Vec::with_capacity(64);
        for rank in 0..8 {
            for file in 0..8 {
                args.push((rank, file));
            }
        }
        let mut rng = thread_rng();
        args.shuffle(&mut rng);
        b.iter(|| {
            for (r, f) in &args {
                black_box(Locale::new(*r, *f));
            }
        });

    }


    #[bench]
    fn benchmark_to_locales_servantlike(b: &mut Bencher) {

        let mut stage = Pinfield(0);
        for r in 0..8 {
            stage = stage.alight(Locale::new(1, r));
        }

        b.iter(|| {
            for _ in 0..100 {
                black_box(stage.to_locales());
            }
        });
    }

    #[bench]
    fn benchmark_to_locales_scholarlike(b: &mut Bencher) {
        let mut stage = Pinfield(0);
        stage = stage.alight(Locale::new(0, 2));
        stage = stage.alight(Locale::new(0, 5));

        b.iter(|| {
            for _ in 0..100 {
                black_box(stage.to_locales());
            }
        });
    }

    #[bench]
    fn benchmark_transit(b: &mut Bencher) {
        let mut stage = Pinfield(0);
        let from = Locale::new(1, 3);
        let to = Locale::new(2, 5);
        stage = stage.alight(from);

        b.iter(|| {
            for _ in 0..10000 {
                black_box(stage.transit(from, to));
            }
        });
    }

    #[test]
    fn concerning_converting_to_algebraic() {
        let actual = iproduct!(0..8, 0..8)
                         .map(|t| Locale::new(t.0, t.1))
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
        let expected = iproduct!(0..8, 0..8).map(|t| Locale::new(t.0, t.1));
        for (expectation, actuality) in expected.zip(ALGEBRAICS.iter()) {
            assert_eq!(expectation,
                       // TODO: again, conversion in iterator
                       Locale::from_algebraic(*actuality));
        }
    }

    #[test]
    fn concerning_all_locales() {
        for rank in 0..8 {
            for file in 0..8 {
                let l = Locale {
                    rank_and_file: (rank << 4) | file
                };
                assert_eq!(rank, l.rank());
                assert_eq!(file, l.file());
            }
        }

    }

    #[test]
    fn test_locale() {
        let a1 = Locale::new(0, 0);
        assert_eq!(Pinfield(1u64), a1.pinpoint());

        let c4 = Locale::new(2, 3);
        assert_eq!(Pinfield(524288u64), c4.pinpoint());
    }

    #[test]
    fn test_displace_locale() {
        assert_eq!(
            Some(Locale::new(3, 5)),
            Locale::new(4, 4).displace((-1, 1))
        );
        assert_eq!(
            None,
            Locale::new(0, 0).displace((-1, 1))
        )
    }

    #[test]
    fn concerning_multidisplacement() {
        for i in 0..8 {
            assert_eq!(
                Some(Locale::new(i as u8, i as u8 )),
                Locale::new(0, 0).multidisplace((1, 1), i)
            )
        }
        assert_eq!(None, Locale::new(0, 0).multidisplace((1, 1), 8));
    }


    #[test]
    fn test_alight_and_quench() {
        let mut stage = Pinfield(0);
        let b5 = Locale::new(1, 4);
        stage = stage.alight(b5);
        assert!(stage.query(b5));
        stage = stage.quench(b5);
        assert!(!stage.query(b5));
    }

    #[test]
    fn test_init_pinfield() {
        let starters = vec![Locale::new(1, 2),
                            Locale::new(3, 4),
                            Locale::new(5, 6)];
        let stage = Pinfield::init(&starters);
        for &starter in &starters {
            assert!(stage.query(starter));
        }
    }

    #[test]
    fn concerning_pincount() {
        let starters = vec![Locale::new(1, 2),
                            Locale::new(3, 4),
                            Locale::new(5, 6)];
        let stage = Pinfield::init(&starters);
        assert_eq!(3, stage.pincount());
    }
}
