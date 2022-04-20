use {
    super::{create_handler, Queue},
    crate::{lock, Lock},
    console::{Key, Term},
    std::{
        io::{
            ErrorKind::{ConnectionAborted, ConnectionReset, TimedOut},
            Read, Result, Write,
        },
        net::TcpStream,
        sync::Arc,
        thread::JoinHandle,
    },
};

pub struct Tunnel {
    stream_username: String, // The client's username
    stream: Arc<TcpStream>,  // The client
    buffered_stdin: Term,    // A buffered standard input for the main thread
    height: Lock<usize>,     // The current height of the terminal in use
    queue: Lock<Queue>,      // The queue of every message
    msg: Lock<String>,       // The current message
    connected: Lock<bool>,   // Determinate for when the client disconnected
}

impl Tunnel {
    pub fn new(stream_username: String, stream: TcpStream) -> Result<Self> {
        let stream = Arc::new(stream);
        let buffered_stdin = Term::buffered_stdout();
        let height = lock(usize::MIN);
        let queue = lock(Queue::new(height.clone()));
        let msg = lock("".to_owned());
        let connected = lock(true);

        Ok(Self {
            stream_username,
            stream,
            buffered_stdin,
            height,
            queue,
            msg,
            connected,
        })
    }

    fn listener_handle(&self) -> JoinHandle<Result<()>> {
        let thread_stream = self.stream.clone();
        let thread_queue = self.queue.clone();
        let thread_connected = self.connected.clone();

        create_handler(move |cont| -> Result<()> {
            let mut buf = [0; 64];

            if let Err(e) = (&*thread_stream).read(buf.as_mut_slice()) {
                match e.kind() {
                    ConnectionReset | ConnectionAborted | TimedOut => {
                        if let Ok(mut connected) = thread_connected.write() {
                            *connected = false;
                            *cont = false
                        }
                    }
                    _ => println!("Unexpected Error: {}", e.kind()),
                }
            } else {
                if let Ok(mut queue) = thread_queue.write() {
                    queue.enqueue(String::from_utf8_lossy(&buf), "~".to_string(), true);
                }
            }
            Ok(())
        })
    }

    fn writer_handle(&self) -> JoinHandle<Result<()>> {
        let thread_stream_username = self.stream_username.clone();
        let thread_stream = self.stream.clone();
        let thread_height = self.height.clone();
        let thread_queue = self.queue.clone();
        let thread_msg = self.msg.clone();
        let thread_connected = self.connected.clone();

        let mut stdout = Term::buffered_stdout();
        let mut prev = 0;

        create_handler(move |cont| -> Result<()> {
            if let Ok(connected) = thread_connected.read() {
                if *connected {
                    drop(connected);

                    if let Ok(mut height) = thread_height.write() {
                        *height = stdout.size().0 as usize - 1;

                        if prev != *height {
                            stdout.clear_screen()?;
                            stdout.move_cursor_to(0, 0)?;
                            stdout.write_all(thread_stream_username.as_bytes())?;
                        }
                        prev = *height;

                        let mut h = *height - 2;
                        drop(height);

                        if let Ok(mut queue) = thread_queue.write() {
                            for m in queue.iter_mut() {
                                if m.undraft() {
                                    (&*thread_stream).write_all(m.content().as_bytes())?;
                                }
                                stdout.move_cursor_to(0, h)?;
                                stdout.clear_line()?;

                                stdout.write_all(
                                    format!("{} | {}", m.owner(), m.content()).as_bytes(),
                                )?;
                                h -= 1;

                                if h == 1 {
                                    break;
                                }
                            }
                            drop(queue);

                            if let Ok(height) = thread_height.read() {
                                let height = *height;
                                drop(height);

                                if let Ok(msg) = thread_msg.read() {
                                    stdout.move_cursor_to(msg.len(), height)?;
                                    stdout.clear_line()?;
                                    stdout.write_all(msg.as_bytes())?;
                                }
                            }
                        }
                    }
                    stdout.flush()?;
                } else {
                    *cont = false
                }
            }
            Ok(())
        })
    }

    pub fn init(self) -> Result<()> {
        let _listener = self.listener_handle();
        let _writer = self.writer_handle();

        loop {
            if let Ok(k) = self.buffered_stdin.read_key() {
                if let Ok(connected) = self.connected.read() {
                    if *connected {
                        drop(connected);

                        if let Ok(mut msg) = self.msg.write() {
                            match k {
                                Key::Backspace => {
                                    if msg.len() > 0 {
                                        msg.pop();
                                    }
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
                                                queue.enqueue(msg.clone(), "o".to_string(), false);
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
}
