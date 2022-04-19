use super::ConfigError;

#[derive(Clone, Debug)]
pub enum Service {
    Host,
    Client,
}

impl Service {
    pub fn from_str<T: AsRef<str>>(s: T) -> Result<Self, ConfigError> {
        match s.as_ref() {
            "--host" => Ok(Self::Host),
            "--connect" => Ok(Self::Client),
            r @ _ => Err(ConfigError::ServiceError(format!(
                "Unknown service argument provided ({})",
                r
            ))),
        }
    }
}
