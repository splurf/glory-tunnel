use {
    super::{Error, ErrorKind, Result, Service},
    sha2::{Digest, Sha256},
    std::{env::args_os, net::SocketAddr, str::FromStr},
};

//  This is obviously publicized so I might end up making this into an environmental variable
const SALT: &str = "saltmakesfoodtastegood";

fn validate<T>(s: &String, f: impl FnOnce(&str) -> T, kind: ErrorKind) -> Result<T> {
    let s = s.trim();
    if s.is_empty() || s == " ".repeat(s.len()) {
        Err(Error::new(kind, "Empty"))
    } else if s.len() > 32 {
        Err(Error::new(kind, "Too long"))
    } else {
        Ok(f(s))
    }
}

#[derive(Debug)]
pub struct Config {
    service: Service,
    address: SocketAddr,
    username: String,
    password: String,
}

impl Config {
    pub fn from_args() -> Result<Self> {
        let args = args_os()
            .map(|a| Some(a.to_str()?.to_string()))
            .collect::<Option<Vec<String>>>()
            .ok_or(Error::new(ErrorKind::Argument, "Invalid argument data"))?;

        if let [service, address, username, password] = &args[1..] {
            Ok(Self {
                service: validate(service, Service::from_str, ErrorKind::Service)??,
                address: validate(address, SocketAddr::from_str, ErrorKind::Address)??,
                username: validate(username, ToString::to_string, ErrorKind::Username)?,
                password: validate(
                    password,
                    |s| {
                        let mut hasher = Sha256::new();
                        hasher.update(s);
                        hasher.update(SALT);
                        format!("{:x}", hasher.finalize())
                    },
                    ErrorKind::Password,
                )?,
            })
        } else {
            Err(Error::new(
                ErrorKind::Argument,
                "Invalid number of arguments (5 required)",
            ))
        }
    }

    pub fn service(&self) -> &Service {
        &self.service
    }

    pub fn address(&self) -> &SocketAddr {
        &self.address
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }
}
