# Ascii Pack
This is a simple proc macro library for serializing/deserializing strictly sized ascii text formats in rust, with interpolation to the intended type.

## Example
```rust
#[derive(AsciiPack, PartialEq, Eq, Debug, Default)]
struct TestFormat {
    #[pack(size = 4)]
    pub padded_number: u32,

    #[pack_ignore]
    pub ignored_field: Option::<usize>,

    #[pack(size = 6, pad_left = ' ')]
    pub handling: String,

    #[pack(size = 2)]
    pub line_ending1: String,

    #[pack(size = 9)]
    pub nested_struct: Inner,

    #[pack(size = 10)]
    pub timestamp: u64,

    #[pack(size = 1)]
    pub spacer: char,

    #[pack_vec(size = 4, until = until::empty)]
    pub vec: Vec<String>,
}

#[derive(AsciiPack, PartialEq, Eq, Debug, Default)]
pub struct Inner {
    #[pack(size = 5, pad_left = ' ')]
    pub my_string: String,

    #[pack(size = 4)]
    pub my_number: usize
}

[#test]
fn test() {
    const TEST_ASCII: &str = "0012  TEST\r\n0123INNER01231697774260 001004143321";

    // converting from the ascii format into a struct
    let unpacked = TestFormat::from_ascii(TEST_ASCII).unwrap();

    assert_eq!(unpacked.padded_number, 12);
    assert_eq!(unpacked.line_ending1, "\r\n");
    
    // ...

    // converting back to the packed ascii format
    assert_eq!(unpacked.to_ascii().unwrap(), TEST_ASCII);
}
```