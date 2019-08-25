use std::{
    error,
    fmt::{Display, Formatter, Result as FmtResult},
    io::{self, ErrorKind},
};

#[derive(Debug)]
pub enum Error {
    Custom(String),

    LengthRequired,

    UnexpectedEndOfBuffer,

    Unsupported,

    #[doc(hidden)]
    __NonExhaustive,
}

impl error::Error for Error {}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::Custom(msg.to_string())
    }
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::Custom(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match self {
            Error::Custom(msg) => fmt.write_str(msg),
            Error::LengthRequired => {
                write!(fmt, "serializing sequences requires specifying a length")
            }
            Error::UnexpectedEndOfBuffer => write!(fmt, "unexpected end of buffer"),
            Error::Unsupported => write!(fmt, "unsupported data type"),
            Error::__NonExhaustive => write!(fmt, "unknown"),
        }
    }
}

impl From<Error> for io::Error {
    fn from(e: Error) -> Self {
        match e {
            Error::Custom(v) => io::Error::new(ErrorKind::Other, v),
            Error::LengthRequired => {
                io::Error::new(ErrorKind::InvalidData, "length required")
            }
            Error::UnexpectedEndOfBuffer => ErrorKind::UnexpectedEof.into(),
            Error::Unsupported => io::Error::new(ErrorKind::Other, "unsupported"),
            _ => ErrorKind::Other.into(),
        }
    }
}
