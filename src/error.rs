
use std::io;
use std::path;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    Io{source: io::Error, path: path::PathBuf, message: String},
    TranscodeError(TranscodeError),
}

impl Error {
    pub fn other(message: &str) -> Error {
        return Error::TranscodeError(TranscodeError::Other(message.into()));
    }
}

impl From<TranscodeError> for Error {
    fn from(e:TranscodeError) -> Self {
        return Error::TranscodeError(e);
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io{source, path, message} => write!(f, "{}: {}. Cause: {}", path.to_string_lossy(), message, source),
            Error::TranscodeError(err) => err.fmt(f),
        }
    }
}

#[derive(Debug)]
pub enum TranscodeError {
    Read(io::Error),
    Write(io::Error),
    Guess(String),
    Other(String),
}

impl std::error::Error for TranscodeError {}

impl fmt::Display for TranscodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TranscodeError::Read(error)|TranscodeError::Write(error) => write!(f, "{}", error),
            TranscodeError::Guess(message)|TranscodeError::Other(message) => write!(f, "{}", message),
        }
    }
}
