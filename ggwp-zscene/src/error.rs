use std::{error, fmt};

#[derive(Debug)]
pub enum Error {
    GgezError(ggez::GameError),
    NoDimensions,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::GgezError(ref e) => write!(f, "GGEZ Error: {}", e),
            Error::NoDimensions => write!(f, "The drawable has no dimensions"),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::GgezError(ref e) => Some(e),
            Error::NoDimensions => None,
        }
    }
}

impl From<ggez::GameError> for Error {
    fn from(e: ggez::GameError) -> Self {
        Error::GgezError(e)
    }
}
