
use std::io;
use std::path;
use std::fmt;
use exitcode;

#[derive(Debug)]
pub enum Error {
    Io{source: io::Error, path: path::PathBuf, message: String},
    BrokenPipe,
    Guess(String),
    Usage(String),
}

impl Error {
    pub fn is_guess(self: &Self) -> bool {
        if let Error::Guess(_) = self {
            return true;
        }
        return false;
    }
    pub fn is_broken_pipe(self: &Self) -> bool {
        if let Error::BrokenPipe = self {
            return true;
        }
        return false;
    }
    pub fn error_code(self: &Self) -> exitcode::ExitCode {
        match self {
            Error::Io{..} => exitcode::IOERR,
            Error::BrokenPipe => exitcode::OK, // Ignore broken pipe error. rust-lang/rust#46016
            Error::Guess(_) => exitcode::DATAERR,
            Error::Usage(_) => exitcode::USAGE,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io{source, path, message} => write!(f, "{}: {}. Cause: {}", path.to_string_lossy(), message, source),
            Error::BrokenPipe => write!(f, ""), // Ignore broken pipe error. rust-lang/rust#46016
            Error::Guess(message)|Error::Usage(message) => write!(f, "{}", message),
        }
    }
}
