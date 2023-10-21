use std::{convert::Infallible, num::ParseIntError, str::FromStr};
use thiserror::Error;

pub use ascii_pack_macro::*;

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
    #[error("Infallible!")]
    Infallible(#[from] Infallible),
}

impl<T> AsciiPack for T
where
    T: FromStr + ToString,
    AsciiPackError: From<<T as FromStr>::Err>,
{
    fn from_ascii(input: &str) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self::from_str(input)?)
    }

    fn to_ascii(&self) -> Result<String> {
        Ok(self.to_string())
    }
}
