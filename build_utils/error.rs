/// Error type for build operations
#[derive(Debug)]
pub enum BuildError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Template(String),
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildError::Io(e) => write!(f, "IO error: {}", e),
            BuildError::Json(e) => write!(f, "JSON error: {}", e),
            BuildError::Template(msg) => write!(f, "Template error: {}", msg),
        }
    }
}

impl std::error::Error for BuildError {}

impl From<std::io::Error> for BuildError {
    fn from(err: std::io::Error) -> Self {
        BuildError::Io(err)
    }
}

impl From<serde_json::Error> for BuildError {
    fn from(err: serde_json::Error) -> Self {
        BuildError::Json(err)
    }
}
