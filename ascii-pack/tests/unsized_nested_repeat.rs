// this represents the biggest pain in the ass use case
// of AsciiPack - nesting multiple structures of unknown
// size inside a structure of therefore unknown size.

use ascii_pack::until;
use ascii_pack::AsciiPack;
use ascii_pack::AsciiPackError;
use ntest_timeout::timeout;

// Note: the formatting here is intentional
const EXAMPLE: &str = "RECORD:
00 0501//0443//0125//1064//
01 0045//0002//0073//0234//
02 0291//0342//2303//0974//
";

/// `0012//`
#[derive(AsciiPack, PartialEq, Eq, Debug, Default)]
pub struct Item {
    #[pack(size = 4)]
    pub id: usize,

    #[pack(size = 2)]
    pub _s_delimeter: String,
}

/// `00 0001//0002//0003//0004//\n`
#[derive(AsciiPack, PartialEq, Eq, Debug, Default)]
pub struct Day {
    #[pack(size = 2)]
    pub day: usize,

    #[pack(size = 1)]
    pub _spacer: char,

    #[pack_vec(until = until::starts_with("\n"))]
    pub vec: Vec<Item>,

    #[pack(size = 1)]
    pub _newline: char,
}

/// `RECORD:\n`
/// `00 0501//0443//0125//1064//\n`
/// `01 0045//0002//0073//0234//\n`
/// `02 0291//0342//2303//0974//\n`
#[derive(AsciiPack, PartialEq, Eq, Debug, Default)]
pub struct MultipleDays {
    // TODO: #[pack_static(text = "RECORD:\n")]
    #[pack(size = 8)]
    pub _record: String,

    #[pack_vec(until = until::empty)]
    pub days: Vec<Day>,
}

#[test]
#[timeout(3000)]
fn test_nested_repeat() {
    let record = MultipleDays::from_ascii(EXAMPLE).unwrap();

    let repacked = record.to_ascii().unwrap();

    assert_eq!(EXAMPLE, repacked);
}
