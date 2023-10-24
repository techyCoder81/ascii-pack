use ascii_pack::AsciiPack;
use ascii_pack::AsciiPackError;

#[derive(AsciiPack, PartialEq, Eq, Debug, Default)]
pub struct Inner {
    #[pack(size = 6, pad_left = ' ')]
    pub my_string: String,

    #[pack(size = 4)]
    pub my_number: usize,
}

#[derive(AsciiPack, PartialEq, Eq, Debug, Default)]
pub struct Outer {
    #[pack(size = 4)]
    pub field1: u32,

    /// Doc comment
    #[pack(size = 10)]
    pub inner_struct: Inner,
}

#[test]
fn nested_test() {
    let pack = Outer::from_ascii("0123TESTED4567").unwrap();
    assert_eq!(pack.field1, 123);
    assert_eq!(pack.inner_struct.my_number, 4567);
    assert_eq!(pack.inner_struct.my_string, "TESTED");
}
