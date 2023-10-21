#![feature(never_type)]

use ascii_pack::AsciiPack;
use std::str::FromStr;

#[derive(Debug, Default, Eq, PartialEq)]
struct PrimitiveBool(bool);

impl AsciiPack for PrimitiveBool {
    fn from_ascii(input: &str) -> ascii_pack::Result<Self>
    where
        Self: Sized,
    {
        match input {
            "0" => Ok(PrimitiveBool(false)),
            "1" => Ok(PrimitiveBool(true)),
            other => Err(AsciiPackError::Unpack(
                "failed to parse primitive bool!".to_owned(),
            )),
        }
    }

    fn to_ascii(&self) -> ascii_pack::Result<String> {
        match self {
            PrimitiveBool(true) => Ok("1".to_owned()),
            PrimitiveBool(false) => Ok("0".to_owned()),
        }
    }
}

#[derive(AsciiPack, PartialEq, Eq, Debug, Default)]
struct TestFormat {
    #[pack(size = 4)]
    pub padded_number: u32,

    #[pack_ignore]
    pub ignored_field: Option<usize>,

    #[pack(size = 6, pad_left = ' ')]
    pub handling: String,

    #[pack(size = 1)]
    pub flag: PrimitiveBool,

    #[pack(size = 2)]
    pub line_ending1: String,

    #[pack(size = 10)]
    pub timestamp: u64,
}

#[test]
fn simple_test() {
    const TEST_ASCII: &str = "0012  TEST1\r\n1697774260";
    let unpacked = TestFormat::from_ascii(TEST_ASCII).unwrap();

    assert_eq!(unpacked.padded_number, 12);
    assert_eq!(unpacked.handling, "  TEST");
    assert_eq!(unpacked.flag, PrimitiveBool(true));
    assert_eq!(unpacked.line_ending1, "\r\n");
    assert_eq!(unpacked.timestamp, 1697774260);

    // pack_ignore uses the default() implementation for the field.
    assert_eq!(unpacked.ignored_field, None);

    // the struct should pack back into the same string
    assert_eq!(unpacked.to_ascii().unwrap(), TEST_ASCII);

    // ToString and FromStr should thinly wrap `to_ascii()` and `from_ascii()`
    assert_eq!(TestFormat::from_ascii(TEST_ASCII).unwrap(), unpacked);
    assert_eq!(unpacked.to_ascii().unwrap(), unpacked.to_ascii().unwrap());
}

mod nested;

#[test]
fn nested_test() {
    let pack = nested::outer::Outer::from_ascii("0123TESTED4567").unwrap();
    assert_eq!(pack.field1, 123);
    assert_eq!(pack.inner_struct.my_number, 4567);
    assert_eq!(pack.inner_struct.my_string, "TESTED");
}
