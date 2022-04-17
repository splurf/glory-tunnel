use {
    crate::{Config, Service, Tunnel},
    std::{
        io::{Read, Result, Write},
        net::TcpStream,
    },
};

type RawScript = Box<dyn FnOnce() -> Result<()>>;

pub struct Script(RawScript);

impl Script {
    pub fn run(self) -> Result<()> {
        self.0()
    }
}

impl<T: FnOnce() -> Result<()> + 'static> From<T> for Script {
    fn from(raw: T) -> Self {
        Self(Box::new(raw))
    }
}

impl From<Config> for Script {
    fn from(config: Config) -> Self {
        match config.service() {
            Service::Host => Script::from(move || {
                use std::net::{Shutdown, TcpListener};

                let server = TcpListener::bind(config.address())?;

                let accept =
                    |mut stream: TcpStream, password: String| -> Result<Option<TcpStream>> {
                        // The size of a Sha256 hash
                        let mut buf = [0; 64];

                        // The stream will alwayswrite the hash of their provided password, meaning, the size of their content will always be 64 bytes
                        stream.read_exact(&mut buf)?;

                        let valid = buf == password.as_bytes();

                        stream.write_all(&[valid as u8; 1])?;

                        Ok(if valid {
                            Some(stream)
                        } else {
                            stream.shutdown(Shutdown::Both)?;
                            None
                        })
                    };

                for stream in server.incoming().filter_map(Result::ok) {
                    if let Ok(Some(mut stream)) = accept(stream, config.password()) {
                        let mut buf = [0; 64];
                        stream.read(&mut buf)?;

                        stream.write_all(config.username().as_bytes())?;

                        let tunnel =
                            Tunnel::new(String::from_utf8_lossy(&buf).to_string(), stream)?;
                        tunnel.init()?;
                        break;
                    }
                }
                Ok(())
            }),
            Service::Client => Script::from(move || {
                let mut stream = TcpStream::connect(config.address())?;
                stream.write_all(config.password().as_bytes())?;

                let mut buf = [0; 1];
                stream.read_exact(&mut buf)?;

                if buf[0] == 1 {
                    let buf = config.username();
                    stream.write_all(buf.as_bytes())?;

                    let mut buf = [0; 64];
                    stream.read(&mut buf)?;

                    let tunnel = Tunnel::new(String::from_utf8_lossy(&buf).to_string(), stream)?;
                    tunnel.init()?;
                } else {
                    println!("Incorrect Password")
                }
                Ok(())
            }),
        }
    }
}
