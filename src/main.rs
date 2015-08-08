
#[derive(Eq,PartialEq,Debug)]
struct Bitboard(u64);

#[derive(Eq,PartialEq,Debug)]
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

#[test]
fn test_locale() {
    let a1 = Locale { rank: 0, file: 0 };
    assert_eq!(Bitboard(1u64), a1.pinpoint());

    let c4 = Locale { rank: 2, file: 3 };
    assert_eq!(Bitboard(524288u64), c4.pinpoint());
}
