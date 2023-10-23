use ascii_pack::{until, AsciiPack, AsciiPackError, Static};

#[derive(AsciiPack, PartialEq, Eq, Debug, Default)]
struct TestFormat {
    #[pack(size = 4)]
    pub padded_number: u32,

    #[pack_ignore]
    pub ignored_field: Option<usize>,

    #[pack(size = 6, pad_left = ' ')]
    pub kind: String,

    #[pack_static(text = "\r\n")]
    pub line_ending1: Static,

    #[pack(size = 9)]
    pub nested_struct: Inner,

    #[pack(size = 10)]
    pub timestamp: u64,

    #[pack_static(text = " ")]
    pub spacer: Static,

    #[pack_vec(size = 4, until = until::empty)]
    pub vec: Vec<String>,
}

#[derive(AsciiPack, PartialEq, Eq, Debug, Default)]
pub struct Inner {
    #[pack(size = 5, pad_left = ' ')]
    pub my_string: String,

    #[pack(size = 4)]
    pub my_number: usize,
}

#[test]
fn complex_example() {
    const TEST_ASCII: &str = "0012  TEST\r\nINNER01231697774260 001004143321";

    // converting from the ascii format into a struct
    let unpacked = TestFormat::from_ascii(TEST_ASCII).unwrap();

    assert_eq!(unpacked.padded_number, 12);
    assert_eq!(unpacked.timestamp, 1697774260);
    assert_eq!(unpacked.vec.len(), 3);
    assert_eq!(unpacked.nested_struct.my_string, "INNER");
    assert_eq!(unpacked.nested_struct.my_number, 123);
    assert_eq!(unpacked.kind, "  TEST");

    // converting back to the packed ascii format
    assert_eq!(unpacked.to_ascii().unwrap(), TEST_ASCII);
}
