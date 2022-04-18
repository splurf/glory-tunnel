use std::{
    io::{Read, Result, Write},
    net::TcpStream,
};

use crate::Tunnel;

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

            let address = SocketAddr::from_str(&args[2]).ok()?;

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

    pub fn run(self) -> Result<()> {
        match self.service() {
            Service::Host => {
                use std::net::{Shutdown, TcpListener};

                let server = TcpListener::bind(self.address())?;

                let accept = |mut stream: TcpStream| -> Result<Option<TcpStream>> {
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

                for stream in server.incoming().filter_map(Result::ok) {
                    if let Ok(Some(mut stream)) = accept(stream) {
                        //  Receive the client's username
                        let mut buf = [0; 64];
                        stream.read(&mut buf)?;

                        // Send the host's username
                        stream.write_all(self.username().as_bytes())?;

                        //  Initiate tunnel connection
                        let tunnel =
                            Tunnel::new(String::from_utf8_lossy(&buf).to_string(), stream)?;
                        tunnel.init()?;
                        break;
                    }
                }
                Ok(())
            }
            Service::Client => {
                let mut stream = TcpStream::connect(self.address())?;
                stream.write_all(self.password().as_bytes())?;

                let mut buf = [0; 1];
                stream.read_exact(&mut buf)?;

                if buf[0] == 1 {
                    // Send the client's username
                    stream.write_all(self.username().as_bytes())?;

                    //  Receive the host's username
                    let mut buf = [0; 64];
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
