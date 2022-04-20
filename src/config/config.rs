use {
    super::{ConfigError, Service},
    crate::Tunnel,
    sha2::{Digest, Sha256},
    std::{
        env::args_os,
        io::{Error, Read, Write},
        net::{SocketAddr, TcpStream},
        str::FromStr,
    },
};

const SALT: &str = "saltmakesfoodtastegood";

#[derive(Debug)]
#[non_exhaustive]
pub struct Config {
    service: Service,
    address: SocketAddr,
    username: String,
    password: String,
}

impl Config {
    pub fn from_args() -> Result<Self, ConfigError> {
        let args = args_os()
            .map(|a| a.to_string_lossy().to_string())
            .collect::<Vec<String>>();

        if let [service, address, username, password] = &args[1..] {
            Ok(Self {
                service: Service::from_str(service.trim())?,
                address: SocketAddr::from_str(address.trim())?,
                username: {
                    let s = username.trim();
                    if s.is_empty() || s == " ".repeat(s.len()) {
                        Err(ConfigError::UsernameError("Username is empty".to_string()))
                    } else if s.len() > 32 {
                        Err(ConfigError::UsernameError(
                            "Username is too long (32 character limit)".to_string(),
                        ))
                    } else {
                        Ok(s.to_string())
                    }
                }?,
                password: {
                    let p = password.trim();
                    let mut hasher = Sha256::new();
                    hasher.update(p);
                    hasher.update(SALT);
                    format!("{:x}", hasher.finalize())
                },
            })
        } else {
            Err(ConfigError::ArgumentError(
                "Invalid number of arguments (5 required)".to_string(),
            ))
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

    pub fn run(self) -> Result<(), Error> {
        match self.service() {
            Service::Host => {
                use std::net::{Shutdown, TcpListener};

                let server = TcpListener::bind(self.address())?;

                let accept = |mut stream: TcpStream| -> Result<Option<TcpStream>, Error> {
                    // The size of a Sha256 hash
                    let mut buf = [0; 64];

                    // The stream will always send the hash of their provided password, meaning, the size of their content will always be 64 bytes
                    stream.read_exact(&mut buf)?;

                    let valid = buf == self.password().as_bytes();

                    // To prevent from having a `write_all` statement for each condition below
                    stream.write_all(&[valid as u8; 1])?;

                    Ok(if valid {
                        Some(stream)
                    } else {
                        stream.shutdown(Shutdown::Both)?;
                        None
                    })
                };

                println!("Waiting for incoming connection...\n");

                for stream in server.incoming().filter_map(Result::ok) {
                    print!("{:?} : ", stream.local_addr()?);

                    if let Ok(Some(mut stream)) = accept(stream) {
                        println!("Done");

                        //  Receive the client's username
                        let mut buf = [0; 32];
                        stream.read(&mut buf)?;

                        // Send the host's username
                        stream.write_all(self.username().as_bytes())?;

                        //  Initiate tunnel connection
                        let tunnel =
                            Tunnel::new(String::from_utf8_lossy(&buf).to_string(), stream)?;
                        tunnel.init()?;
                        break;
                    } else {
                        println!("Incorrect Password")
                    }
                }
                Ok(())
            }
            Service::Client => {
                println!("Attempting to connect...");

                let mut stream = TcpStream::connect(self.address())?;
                stream.write_all(self.password().as_bytes())?;

                let mut buf = [0; 1];
                stream.read_exact(&mut buf)?;

                if buf[0] == 1 {
                    println!("Done");

                    // Send the client's username
                    stream.write_all(self.username().as_bytes())?;

                    //  Receive the host's username
                    let mut buf = [0; 32];
                    stream.read(&mut buf)?;

                    //  Initiate tunnel connection
                    let tunnel = Tunnel::new(String::from_utf8_lossy(&buf).to_string(), stream)?;
                    tunnel.init()?;
                } else {
                    println!("Incorrect Password")
                }
                Ok(())
            }
        }
    }
}
