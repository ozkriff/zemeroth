use std::{error, fmt};

#[derive(Debug)]
pub enum Error {
    GwgError(gwg::GameError),
    NoDimensions,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::GwgError(ref e) => write!(f, "gwg Error: {}", e),
            Error::NoDimensions => write!(f, "The drawable has no dimensions"),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::GwgError(ref e) => Some(e),
            Error::NoDimensions => None,
        }
    }
}

impl From<gwg::GameError> for Error {
    fn from(e: gwg::GameError) -> Self {
        Error::GwgError(e)
    }
}
