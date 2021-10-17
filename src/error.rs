
use std::io;
use std::path;
use std::fmt;

#[derive(Debug)]
pub enum Error<> {
    Io{source: io::Error, path: path::PathBuf, message: String},
    Other(String),
}

trait New<T> {
    fn new() -> T;
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io{source, path, message}
                => write!(f, "{}. {}\nCaused By: {}", message, path.to_string_lossy(), source),
            Error::Other(message) => write!(f, "{}", message),
        }
    }
}
