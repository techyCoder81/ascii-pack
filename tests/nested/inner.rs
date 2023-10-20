use ascii_pack::AsciiPack;

#[derive(AsciiPack, PartialEq, Eq, Debug, Default)]
pub struct Inner {
    #[pack(size = 6, pad_left = ' ')]
    pub my_string: String,

    #[pack(size = 4)]
    pub my_number: usize
}