pub use ascii_pack_macro::*;

#[derive(Debug)]
pub struct MessageFormatParseError {
    pub error: String
}

pub trait AsciiPack {
    fn from_ascii(input: &str) -> Result<Self, MessageFormatParseError> where Self: Sized;
    fn to_ascii(&self) -> Result<String, MessageFormatParseError>;
}