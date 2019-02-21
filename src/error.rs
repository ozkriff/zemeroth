use std::{error, fmt, io, path::PathBuf};

#[derive(Debug)]
pub enum ZError {
    GgezError(ggez::GameError),
    UiError(ui::Error),
    SceneError(scene::Error),
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
            ZError::GgezError(ref e) => write!(f, "GGEZ Error: {}", e),
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
            ZError::GgezError(ref e) => Some(e),
            ZError::UiError(ref e) => Some(e),
            ZError::SceneError(ref e) => Some(e),
            ZError::RonDeserializeError { error, .. } => Some(error),
            ZError::IOError(ref e) => Some(e),
        }
    }
}

impl From<ggez::GameError> for ZError {
    fn from(e: ggez::GameError) -> Self {
        ZError::GgezError(e)
    }
}

impl From<ui::Error> for ZError {
    fn from(e: ui::Error) -> Self {
        ZError::UiError(e)
    }
}

impl From<scene::Error> for ZError {
    fn from(e: scene::Error) -> Self {
        ZError::SceneError(e)
    }
}

impl From<io::Error> for ZError {
    fn from(e: io::Error) -> ZError {
        ZError::IOError(e)
    }
}
