use std::fmt;
use std::io;

#[derive(Debug)]
pub enum Error {
    ShellNotFound,
    HomeNotFound,
    ConfigParseError,
    ExecutionError,
    IoError(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::ShellNotFound => write!(f, "Shell Not Found"),
            Error::HomeNotFound => write!(f, "home Not Found"),
            Error::ConfigParseError => write!(f, "Config Parse Error"),
            Error::ExecutionError => write!(f, "Execution Error"),
            Error::IoError(ref err) => write!(f, "IO Error: {}", err),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}
