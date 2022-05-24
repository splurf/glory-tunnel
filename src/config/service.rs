use super::{Error, ErrorKind, Result};

#[derive(Debug)]
pub enum Service {
    Host,
    Client,
}

impl Service {
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "--host" => Ok(Self::Host),
            "--connect" => Ok(Self::Client),
            _ => Err(Error::new(
                ErrorKind::Service,
                format!("Unknown service argument provided ({})", s),
            )),
        }
    }
}
