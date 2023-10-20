# Ascii Pack
This is a simple proc macro library for serializing/deserializing strictly sized ascii text formats in rust, with interpolation to the intended type.

## Example
```rust
#[derive(AsciiPack)]
struct TestFormat {
    #[pack(size = 3)]
    pub number: u32,

    #[pack(size = 6, pad_left = ' ')]
    pub handling: String,

    #[pack(size = 9)]
    pub ip: IpAddr,

    #[pack(size = 3)]
    pub line_ending1: String,

    #[pack(size = 10)]
    pub timestamp: u64,
}

const TEST_ASCII: &str = "012  TEST127.0.0.1\r\n1697774260";
    let unpacked = TestFormat::from_ascii(TEST_ASCII).unwrap();

    assert_eq!(unpacked.number, 12);
    
    assert_eq!(unpacked.to_ascii().unwrap(), TEST_ASCII);
```