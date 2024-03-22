use std::{error, fmt, io, path::PathBuf};

#[derive(Debug, derive_more::From)]
pub enum ZError {
    Ui(ui::Error),
    Scene(zscene::Error),
    RonDeserialize {
        error: ron::de::SpannedError,
        path: PathBuf,
    },
    IO(io::Error),
    Mq(mq::Error),
}

impl ZError {
    pub fn from_ron_de_error(error: ron::de::SpannedError, path: PathBuf) -> Self {
        ZError::RonDeserialize { error, path }
    }
}

impl fmt::Display for ZError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ZError::Ui(ref e) => write!(f, "ZGUI Error: {}", e),
            ZError::Scene(ref e) => write!(f, "ZScene Error: {}", e),
            ZError::RonDeserialize { error, path } => {
                let s = path.to_str().unwrap_or("<no path>");
                write!(f, "Can't deserialize '{}': {}", s, error)
            }
            ZError::IO(ref e) => write!(f, "IO Error: {}", e),
            ZError::Mq(ref e) => write!(f, "Macroquad error: {}", e),
        }
    }
}

impl error::Error for ZError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            ZError::Ui(ref e) => Some(e),
            ZError::Scene(ref e) => Some(e),
            ZError::RonDeserialize { error, .. } => Some(error),
            ZError::IO(ref e) => Some(e),
            ZError::Mq(ref e) => Some(e),
        }
    }
}
