use crate::text::{FontError, TexError};
use std::fmt;

/// Every error the library returns, so `?` works everywhere in `main() -> Result`.
#[derive(Debug)]
pub enum Error {
    /// File or ffmpeg problems.
    Io(std::io::Error),
    /// LaTeX could not be compiled or converted.
    Tex(TexError),
    Font(FontError),
    /// Bad input, with an explanation.
    Invalid(String),
}

pub type Result<T = (), E = Error> = std::result::Result<T, E>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "{e}"),
            Error::Tex(e) => write!(f, "{e}"),
            Error::Font(e) => write!(f, "{e}"),
            Error::Invalid(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<TexError> for Error {
    fn from(e: TexError) -> Self {
        Error::Tex(e)
    }
}

impl From<FontError> for Error {
    fn from(e: FontError) -> Self {
        Error::Font(e)
    }
}
