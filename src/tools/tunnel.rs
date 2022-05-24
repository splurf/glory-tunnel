use {
    super::{create_handler, lock, Lock, Queue},
    crate::config::{Config, Service},
    console::{Key, Term},
    std::{
        io::{
            Error,
            ErrorKind::{ConnectionAborted, ConnectionReset, InvalidData, Other, TimedOut},
            Read, Result, Write,
        },
        net::TcpStream,
        sync::Arc,
        thread::JoinHandle,
    },
};

pub struct Tunnel {
    term: Term,             // A buffered standard input for the main thread
    height: Lock<usize>,    // The current height of the terminal in use
    queue: Lock<Queue>,     // The queue of every message
    msg: Lock<String>,      // The current message
    stream: Arc<TcpStream>, // The client's stream
    username: String,       // The client's username
}

impl Tunnel {
    pub fn new(stream: TcpStream, username: String) -> Result<Self> {
        let term = Term::buffered_stdout();
        let height = lock(usize::MIN);
        let queue = lock(Queue::new(height.clone()));
        let msg = lock("".to_owned());
        let stream = Arc::new(stream);

        Ok(Self {
            term,
            height,
            queue,
            msg,
            stream,
            username,
        })
    }

    /**
     * Create a potential `Tunnel` with specified *instructions* when the specified *conditions* are met
     */
    const fn handler(
        conditions: impl Fn(&TcpStream) -> Result<Option<()>>,
        instructions: impl Fn(&TcpStream, &mut [u8; 32]) -> Result<()>,
    ) -> impl Fn(TcpStream) -> Result<Self> {
        move |stream| {
            if let Ok(Some(_)) = conditions(&stream) {
                let mut buf = [0; 32];
                instructions(&stream, &mut buf)?;
                Self::new(
                    stream,
                    String::from_utf8(buf.to_vec()).map_err(|e| Error::new(InvalidData, e))?,
                )
            } else {
                Err(Error::new(InvalidData, "Incorrect Password"))
            }
        }
    }

    fn listener_handle(&self) -> JoinHandle<Result<()>> {
        let thread_stream = self.stream.clone();
        let thread_queue = self.queue.clone();

        create_handler(move |cont| -> Result<()> {
            let mut buf = [0; 64];

            if let Err(e) = (&*thread_stream).read(buf.as_mut_slice()) {
                match e.kind() {
                    ConnectionReset | ConnectionAborted | TimedOut => *cont = false,
                    _ => println!("Unexpected Error: {}", e.kind()),
                }
            } else {
                if let Ok(mut queue) = thread_queue.write() {
                    queue.enqueue(String::from_utf8_lossy(&buf), "~", true);
                }
            }
            Ok(())
        })
    }

    fn writer_handle(&self, listener: JoinHandle<Result<()>>) -> JoinHandle<Result<()>> {
        let mut term = self.term.clone();
        let thread_height = self.height.clone();
        let thread_queue = self.queue.clone();
        let thread_msg = self.msg.clone();
        let thread_stream = self.stream.clone();
        let username = self.username.clone();

        let mut prev = 0;

        create_handler(move |cont| -> Result<()> {
            if !listener.is_finished() {
                if let Ok(mut height) = thread_height.write() {
                    *height = term.size().0 as usize - 1;

                    if prev != *height {
                        term.clear_screen()?;
                        term.move_cursor_to(0, 0)?;
                        term.write_all(username.as_bytes())?;
                    }
                    prev = *height;

                    let mut h = prev - 2;
                    drop(height);

                    if let Ok(mut queue) = thread_queue.write() {
                        for m in queue.iter_mut() {
                            if m.undraft() {
                                (&*thread_stream).write_all(m.content().as_bytes())?;
                            }
                            term.move_cursor_to(0, h)?;
                            term.clear_line()?;
                            term.write_all(format!("{} | {}", m.owner(), m.content()).as_bytes())?;
                            h -= 1;

                            if h == 1 {
                                break;
                            }
                        }
                        drop(queue);

                        if let Ok(height) = thread_height.read() {
                            let y = *height;
                            drop(height);

                            if let Ok(msg) = thread_msg.read() {
                                term.move_cursor_to(msg.len(), y)?;
                                term.clear_line()?;
                                term.write_all(msg.as_bytes())?;
                            }
                        }
                    }
                }
                term.flush()?;
            } else {
                term.clear_line()?;
                term.write_all(b"Disconnected")?;
                term.flush()?;
                *cont = false
            }
            Ok(())
        })
    }

    pub fn init(self) -> Result<()> {
        let listener = self.listener_handle();
        let writer = self.writer_handle(listener);

        loop {
            if let Ok(k) = self.term.read_key() {
                if !writer.is_finished() {
                    if let Ok(mut msg) = self.msg.write() {
                        match k {
                            Key::Backspace => {
                                msg.pop();
                            }
                            Key::Char(c) => {
                                if msg.len() < 64 && (!msg.is_empty() || !c.is_whitespace()) {
                                    msg.push(c)
                                }
                            }
                            Key::Enter => {
                                if !msg.is_empty() {
                                    if *msg == "exit" {
                                        break Ok(());
                                    } else {
                                        if let Ok(mut queue) = self.queue.write() {
                                            queue.enqueue(msg.clone(), "o", false);
                                            msg.clear();
                                        }
                                    }
                                }
                            }
                            _ => (),
                        }
                    }
                } else {
                    break Ok(());
                }
            }
        }
    }
}

impl TryFrom<Config> for Tunnel {
    type Error = Error;

    fn try_from(config: Config) -> Result<Self> {
        match config.service() {
            Service::Host => {
                use std::net::{Shutdown, TcpListener};

                let server = TcpListener::bind(config.address())?;

                println!("TCP: Listening on {}...\n", config.address());

                //  Handler to create potential tunnel
                let handler = Tunnel::handler(
                    |mut stream: &TcpStream| -> Result<Option<()>> {
                        print!("{} : ", stream.local_addr()?);

                        // The size of a Sha256 hash
                        let mut buf = [0; 64];

                        // The stream will always send the hash of their provided password, meaning, the size of their content will always be 64 bytes
                        stream.read_exact(&mut buf)?;

                        let valid = buf == config.password().as_bytes();

                        // To prevent from having a `write_all` statement for each condition below
                        stream.write_all(&[valid as u8; 1])?;

                        Ok(if valid {
                            Some(())
                        } else {
                            stream.shutdown(Shutdown::Both)?;
                            None
                        })
                    },
                    |mut stream, buf| {
                        stream.read(buf)?;

                        // Send the host's username
                        stream.write_all(config.username().as_bytes())
                    },
                );
                let mut tunnel = None;

                for stream in server.incoming().filter_map(Result::ok) {
                    match handler(stream) {
                        Ok(raw) => {
                            println!("Authenticated");
                            tunnel = Some(raw);
                            break;
                        }
                        Err(e) => println!("{}", e),
                    }
                }
                tunnel.ok_or(Other.into())
            }

            Service::Client => {
                println!("Connecting: {}", config.address());
                let mut stream = TcpStream::connect(config.address())?;

                println!("Authenticating");
                //  Send the password to the host
                stream.write_all(config.password().as_bytes())?;
                //  Receive validation
                let mut buf = [0; 1];
                stream.read_exact(&mut buf)?;

                //  Handler to create potential tunnel
                let handler = Tunnel::handler(
                    |_| Ok(if buf[0] == 1 { Some(()) } else { None }),
                    |mut stream, buf| {
                        // Send the client's username
                        stream.write_all(config.username().as_bytes())?;

                        //  Receive the host's username
                        stream.read(buf)?;
                        Ok(())
                    },
                );
                handler(stream)
            }
        }
    }
}
