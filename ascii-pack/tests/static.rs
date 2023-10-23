use ascii_pack::{AsciiPack, AsciiPackError, Static};

const EXAMPLE: &str = "BEGIN1234END";

#[derive(AsciiPack, PartialEq, Eq, Debug, Default)]
pub struct StaticTest {
    #[pack_static(text = "BEGIN")]
    pub begin: Static,

    #[pack(size = 4)]
    pub number: usize,

    #[pack_static(text = "END")]
    pub end: Static,
}

#[test]
fn static_delimeters() {
    let packed = StaticTest::from_ascii(EXAMPLE).unwrap();
    assert_eq!(packed.number, 1234);
    assert_eq!(packed.to_ascii().unwrap(), EXAMPLE);
}
