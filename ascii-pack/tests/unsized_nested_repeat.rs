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
R001004143321";

/// `0012//`
#[derive(AsciiPack, PartialEq, Eq, Debug, Default)]
pub struct Item {
    #[pack(size = 4)]
    pub id: usize,

    #[pack(size = 2)]
    pub _delimeter: String,
}

/// `00 0001//0002//0003//0004//\n`
#[derive(AsciiPack, PartialEq, Eq, Debug, Default)]
pub struct Day {
    #[pack(size = 2)]
    pub day_num: usize,

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
    #[pack_static(text = "RECORD:\n")]
    pub _record: String,

    #[pack_vec(until = until::ascii_alpha)]
    pub days: Vec<Day>,

    #[pack(size = 1)]
    pub _r: char,

    #[pack_vec(size = 3, until = until::empty)]
    pub end_list: Vec<String>,
}

#[test]
#[timeout(3000)]
fn test_unsized_nested_repeat() {
    let record = MultipleDays::from_ascii(EXAMPLE).unwrap();

    assert_eq!(record.days.len(), 3);
    let mut num = 0;
    for day in record.days.iter() {
        assert_eq!(day.day_num, num);
        assert_eq!(day.vec.len(), 4);
        num += 1;
    }

    assert_eq!(record.end_list.len(), 4);

    let repacked = record.to_ascii().unwrap();
    assert_eq!(EXAMPLE, repacked);
}
