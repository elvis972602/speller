use std::io;

#[derive(Debug)]
pub enum BuildError {
    IoError(io::Error),
    #[cfg(feature = "serde_json")]
    JsonError(serde_json::Error),
    #[cfg(feature = "csv")]
    CSVError(csv::Error),
    FileTypeNotSupported,
    NotJsonFile,
    CSVIndexError,
    TXTIndexError,
    ParseCountError,
    DictNotFound,
}

impl From<io::Error> for BuildError {
    fn from(error: io::Error) -> Self {
        BuildError::IoError(error)
    }
}

#[cfg(feature = "serde_json")]
impl From<serde_json::Error> for BuildError {
    fn from(error: serde_json::Error) -> Self {
        BuildError::JsonError(error)
    }
}

#[cfg(feature = "csv")]
impl From<csv::Error> for BuildError {
    fn from(error: csv::Error) -> Self {
        BuildError::CSVError(error)
    }
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BuildError::IoError(e) => write!(f, "IO error: {e}"),
            #[cfg(feature = "serde_json")]
            BuildError::JsonError(e) => write!(f, "JSON error: {e}"),
            #[cfg(feature = "csv")]
            BuildError::CSVError(e) => write!(f, "CSV error: {e}"),
            BuildError::FileTypeNotSupported => write!(f, "File type not supported"),
            BuildError::NotJsonFile => write!(f, "Local dictionary must be a JSON file"),
            BuildError::CSVIndexError => write!(f, "CSV index error"),
            BuildError::TXTIndexError => write!(f, "TXT index error"),
            BuildError::ParseCountError => write!(f, "Error parsing count"),
            BuildError::DictNotFound => write!(f, "Dictionary not found"),
        }
    }
}

impl std::error::Error for BuildError {}
