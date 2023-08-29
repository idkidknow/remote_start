use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    Error(&'static str),
    EnvError {
        variable_name: &'static str
    },
    NetworkError(String),
    IoError(std::io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Error(e) => write!(f, "Error: {}", e),
            Self::EnvError { variable_name } => write!(f, "Invalid environment variable `{}`", variable_name),
            Self::NetworkError(e) => write!(f, "Network error: {}", e),
            Self::IoError(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for Error {}

impl From<&'static str> for Error {
    fn from(value: &'static str) -> Self {
        Self::Error(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}
