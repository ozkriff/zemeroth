use std::{error, fmt, io, path::PathBuf};

#[derive(Debug, derive_more::From)]
pub enum ZError {
    GwgError,
    UiError(ui::Error),
    SceneError(zscene::Error),
    RonDeserializeError {
        error: ron::de::Error,
        path: PathBuf,
    },
    IOError(io::Error),
}

impl ZError {
    pub fn from_ron_de_error(error: ron::de::Error, path: PathBuf) -> Self {
        ZError::RonDeserializeError { error, path }
    }
}

impl fmt::Display for ZError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ZError::GwgError => write!(f, "gwg Error"),
            ZError::UiError(ref e) => write!(f, "ZGUI Error: {}", e),
            ZError::SceneError(ref e) => write!(f, "ZScene Error: {}", e),
            ZError::RonDeserializeError { error, path } => {
                let s = path.to_str().unwrap_or("<no path>");
                write!(f, "Can't deserialize '{}': {}", s, error)
            }
            ZError::IOError(ref e) => write!(f, "IO Error: {}", e),
        }
    }
}

impl error::Error for ZError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            ZError::GwgError => None,
            ZError::UiError(ref e) => Some(e),
            ZError::SceneError(ref e) => Some(e),
            ZError::RonDeserializeError { error, .. } => Some(error),
            ZError::IOError(ref e) => Some(e),
        }
    }
}
