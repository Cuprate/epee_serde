use std::fmt::Display;
use std::io;

use serde::{de, ser};
use thiserror::Error;

use crate::Marker;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Io error: {0}")]
    Io(#[from] io::Error),
    #[error("Invalid Bool value")]
    InvalidBoolValue,
    #[error("Invalid Varint mask")]
    InvalidVarIntMask,
    #[error("Value Marked As String Exceeded Max Length")]
    MarkedStringExceededMaxLength,
    #[error("Invalid String")]
    InvalidString,
    #[error("Invalid Marker: {0}")]
    UnknownMarker(Marker),
    #[error("Unexpected Marker: {0}")]
    UnexpectedMarker(String),
    #[error("Length Mismatch: {0}")]
    LengthMismatch(String),
    #[error("Tuples Of Type {0} Not Supported")]
    TuplesOfTypeNotSupported(Marker),
    #[error("Tuple Structs Not Supported")]
    TupleStructsNotSupported,
    #[error("Missing Header")]
    MissingHeader,
    #[error("Invalid Portable Storage Version: {0}")]
    InvalidVersion(u8),
    #[error("Root must be struct")]
    RootValueIsNotStruct,
    #[error("f32 Not supported")]
    F32NotSupported,
    #[error("Options Not supported")]
    OptionsNotSupported,
    #[error("Unit Not supported")]
    UnitNotSupported,
    #[error("Enums Not supported")]
    EnumNotSupported,
    #[error("Length of Seq/Maps Must Be Known Ahead Of Time")]
    UnknownLength,
    #[error("{0}")]
    Custom(String),
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}

impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::Custom(msg.to_string())
    }
}
