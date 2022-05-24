use std::{io, net::AddrParseError, result};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum ErrorKind {
    Argument,
    Service,
    Address,
    Username,
    Password,
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    error: String,
}

impl Error {
    pub fn new<S: AsRef<str>>(kind: ErrorKind, error: S) -> Self {
        Self {
            kind,
            error: error.as_ref().to_string(),
        }
    }
}

impl From<AddrParseError> for Error {
    fn from(e: AddrParseError) -> Self {
        Self::new(ErrorKind::Address, e.to_string())
    }
}

impl From<Error> for io::Error {
    fn from(e: Error) -> Self {
        Self::new(io::ErrorKind::Other, format!("{:?}: {}", e.kind, e.error))
    }
}
