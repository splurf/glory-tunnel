use std::net::AddrParseError;

#[derive(Debug)]
pub enum ConfigError {
    ArgumentError(String),
    ServiceError(String),
    AddressError(String),
    UsernameError(String),
    PasswordError(String),
}

impl From<AddrParseError> for ConfigError {
    fn from(e: AddrParseError) -> Self {
        Self::AddressError(e.to_string())
    }
}
