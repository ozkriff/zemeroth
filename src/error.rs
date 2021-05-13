use std::{error, fmt, io, path::PathBuf};

#[derive(Debug, derive_more::From)]
pub enum ZError {
    UiError(ui::Error),
    SceneError(zscene::Error),
    RonDeserializeError {
        error: ron::de::Error,
        path: PathBuf,
    },
    IOError(io::Error),
    MqFileError(mq::file::FileError),
    MqFontError(mq::text::FontError),
}

impl ZError {
    pub fn from_ron_de_error(error: ron::de::Error, path: PathBuf) -> Self {
        ZError::RonDeserializeError { error, path }
    }
}

impl fmt::Display for ZError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ZError::UiError(ref e) => write!(f, "ZGUI Error: {}", e),
            ZError::SceneError(ref e) => write!(f, "ZScene Error: {}", e),
            ZError::RonDeserializeError { error, path } => {
                let s = path.to_str().unwrap_or("<no path>");
                write!(f, "Can't deserialize '{}': {}", s, error)
            }
            ZError::IOError(ref e) => write!(f, "IO Error: {}", e),
            ZError::MqFileError(ref e) => write!(f, "Macroquad File error: {}", e),
            ZError::MqFontError(ref e) => write!(f, "Macroquad Font error: {}", e),
        }
    }
}

impl error::Error for ZError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            ZError::UiError(ref e) => Some(e),
            ZError::SceneError(ref e) => Some(e),
            ZError::RonDeserializeError { error, .. } => Some(error),
            ZError::IOError(ref e) => Some(e),
            ZError::MqFileError(ref e) => Some(e),
            ZError::MqFontError(ref e) => Some(e),
        }
    }
}
