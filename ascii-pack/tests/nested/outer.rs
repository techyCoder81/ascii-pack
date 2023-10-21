use ascii_pack::AsciiPack;

#[derive(AsciiPack, PartialEq, Eq, Debug, Default)]
pub struct Outer {
    #[pack(size = 4)]
    pub field1: u32,

    #[pack(size = 10)]
    pub inner_struct: super::inner::Inner
}