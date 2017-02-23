use std;
use std::fmt;
use std::convert::From;
use std::io;


#[derive(Debug)]
pub enum EngineError {
    Io(io::Error),
    UnknownOption(String),
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EngineError::Io(ref err) => write!(f, "IO error: {}", err),
            EngineError::UnknownOption(ref option) => write!(f, "No such option: '{}'", option.as_str())
        }
    }
}

impl std::error::Error for EngineError {
    fn description(&self) -> &str {
        match *self {
            EngineError::Io(ref err) => err.description(),
            EngineError::UnknownOption(..) => "Unknown option"
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            EngineError::Io(ref err) => Some(err),
            EngineError::UnknownOption(..) => None
        }
    }
}

impl From<io::Error> for EngineError {
    fn from(err: io::Error) -> EngineError {
        EngineError::Io(err)
    }
}

pub type Result<T> = std::result::Result<T, EngineError>;
