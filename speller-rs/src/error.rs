use std::io;

#[derive(Debug)]
pub enum BuildError {
    IoError(io::Error),
    JsonError(serde_json::Error),
    NotJsonFile,
}

impl From<io::Error> for BuildError {
    fn from(error: io::Error) -> Self {
        BuildError::IoError(error)
    }
}

impl From<serde_json::Error> for BuildError {
    fn from(error: serde_json::Error) -> Self {
        BuildError::JsonError(error)
    }
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BuildError::IoError(e) => write!(f, "IO error: {}", e),
            BuildError::JsonError(e) => write!(f, "JSON error: {}", e),
            BuildError::NotJsonFile => write!(f, "Local dictionary must be a JSON file"),
        }
    }
}

impl std::error::Error for BuildError {}