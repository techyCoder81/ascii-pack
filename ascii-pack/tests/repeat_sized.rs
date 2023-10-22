use ascii_pack::until;
use ascii_pack::AsciiPack;
use ascii_pack::AsciiPackError;

const TEST_ASCII: &str = "  EXAMPLETESTTESTTEST00120654012346543345delimeterabc";

#[derive(AsciiPack, PartialEq, Eq, Debug, Default)]
pub struct VecTest {
    #[pack(size = 9, pad_left = ' ')]
    pub string1: String,

    #[pack_vec(size = 4, until = until::ascii_digit)]
    pub string_vec: Vec<String>,

    #[pack_vec(size = 4, until = until::starts_with("del"))]
    pub usize_vec: Vec<usize>,

    #[pack(size = 9)]
    pub delimeter: String,

    #[pack_vec(size = 1, until = until::empty)]
    pub trailing_vec: Vec<char>,
}

#[test]
fn test_sized_vec_u8() {
    let result = VecTest::from_ascii(TEST_ASCII);

    let result = match result {
        Ok(result) => result,
        Err(e) => panic!("Error unpacking ascii: {e}"),
    };

    assert_eq!(result.string1, "  EXAMPLE");
    assert_eq!(result.string_vec.len(), 3);
    assert_eq!(result.usize_vec.len(), 5);
    assert_eq!(result.delimeter, "delimeter");
    assert_eq!(result.trailing_vec.len(), 3);

    let repacked = result.to_ascii();
    assert_eq!(repacked.unwrap(), TEST_ASCII);
}
