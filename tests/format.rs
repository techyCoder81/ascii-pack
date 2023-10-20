use std::net::IpAddr;

use ascii_pack::AsciiPack;

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

#[test]
fn test_format() {
    const TEST_ASCII: &str = "012  TEST127.0.0.1\r\r\n1697774260";
    let unpacked = TestFormat::from_ascii(TEST_ASCII).unwrap();

    assert_eq!(unpacked.number, 12);
    assert_eq!(unpacked.handling, "  TEST");
    assert!(unpacked.ip.is_loopback());
    assert_eq!(unpacked.line_ending1, "\r\r\n");
    assert_eq!(unpacked.timestamp, 1697774260);

    assert_eq!(unpacked.to_ascii().unwrap(), TEST_ASCII);
}