use std::{
    char::ParseCharError,
    convert::Infallible,
    num::{ParseFloatError, ParseIntError},
    str::{FromStr, ParseBoolError},
};
use thiserror::Error;

pub use ascii_pack_macro::*;
pub use strum;
pub mod until;

pub type Result<T> = std::result::Result<T, AsciiPackError>;

pub trait AsciiPack {
    fn from_ascii(input: &str) -> Result<Self>
    where
        Self: Sized;
    fn to_ascii(&self) -> Result<String>;
}

#[derive(Error, Debug)]
pub enum AsciiPackError {
    #[error("unknown error: {0}")]
    Unknown(String),
    #[error("Unpacking error: {0}")]
    Unpack(String),
    #[error("Packing error: {0}")]
    Pack(String),
    #[error("parse int failed")]
    ParseIntError(#[from] ParseIntError),
    #[error("parse char failed")]
    ParseCharError(#[from] ParseCharError),
    #[error("parse bool failed")]
    ParseBoolError(#[from] ParseBoolError),
    #[error("parse float failed")]
    ParseFloatError(#[from] ParseFloatError),
    #[error("Infallible")]
    Infallible(#[from] Infallible),
    #[error("Strum parse error")]
    StrumParseError(#[from] strum::ParseError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl<T> AsciiPack for T
where
    T: FromStr + ToString,
    AsciiPackError: From<<T as FromStr>::Err>,
    <T as FromStr>::Err: std::fmt::Debug,
{
    fn from_ascii(input: &str) -> Result<Self>
    where
        Self: Sized,
    {
        let result = Self::from_str(input);

        match result {
            Ok(unpacked) => Ok(unpacked),
            Err(e) => Err(AsciiPackError::Unpack(format!(
                "Error unpacking '{}' : {:?}",
                input, e
            ))),
        }
    }

    fn to_ascii(&self) -> Result<String> {
        Ok(self.to_string())
    }
}

/// This (empty) struct represents a statically-sized ascii field.
/// It's text representation is derived from the `pack_static` attribute
/// assigned to the field definition, and it otherwise contains no data.
#[derive(Default, Eq, PartialEq, Debug, Clone, Copy)]
pub struct Static;

impl Static {
    pub fn new() -> Static {
        Static {}
    }
}
