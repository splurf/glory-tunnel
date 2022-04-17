use {
    super::Service,
    sha2::{Digest, Sha256},
    std::{env::args_os, net::SocketAddr, str::FromStr},
};

const SALT: &str = "toaster repairguy";

#[derive(Debug)]
#[non_exhaustive]
pub struct Config {
    service: Service,
    address: SocketAddr,
    username: String,
    password: String,
}

impl Config {
    pub fn from_args() -> Option<Self> {
        let args = args_os()
            .map(|a| a.to_string_lossy().to_string())
            .collect::<Vec<String>>();

        if args.len() == 5 {
            let service = Service::from_str(&args[1])?;

            //  make this better
            let address = if let Ok(addr) = SocketAddr::from_str(&args[2]) {
                Some(addr)
            } else {
                None
            }?;

            let username = {
                let arg = args[3].trim();
                if !arg.is_empty() && arg.len() < 64 && arg != " ".repeat(arg.len()) {
                    Some(arg.to_string())
                } else {
                    None
                }
            }?;

            let mut hasher = Sha256::new();
            hasher.update(&args[4]);
            hasher.update(SALT);

            let password = format!("{:x}", hasher.finalize());

            Some(Self {
                service,
                address,
                username,
                password,
            })
        } else {
            None
        }
    }

    pub fn service(&self) -> Service {
        self.service.clone()
    }

    pub fn address(&self) -> SocketAddr {
        self.address
    }

    pub fn username(&self) -> String {
        self.username.clone()
    }

    pub fn password(&self) -> String {
        self.password.clone()
    }
}
