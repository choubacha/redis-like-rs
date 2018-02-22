use futures::{Async, Future, Stream, Poll};
use tokio_io::io::WriteHalf;
use tokio_io::AsyncRead;
use tokio::net::TcpStream;
use command::{Command, CommandStream};
use futures::sync::mpsc;
use std::io;
use std::io::Write;
use std::convert::From;
use bytes::Bytes;

pub struct Connection {
    writer: WriteHalf<TcpStream>,
    commands: CommandStream,
    tx: mpsc::Sender<Option<usize>>,
    rx: mpsc::Receiver<Option<usize>>,
}

impl Connection {
    pub fn new(socket: TcpStream) -> Self {
        let (reader, writer) = socket.split();
        let commands = CommandStream::new(reader);
        let (tx, rx) = mpsc::channel(1024);
        Connection { tx, rx, commands, writer }
    }
}

impl Future for Connection {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {
        while let Async::Ready(Some(command)) = self.commands.poll()? {
            println!("command received: {:?}", command);

            macro_rules! write_out {
                ($e:expr) => (
                    {
                        let _ = try_nb!(self.writer.write($e));
                    }
                )
            }

            match command {
                Ok(Command::Get(key)) => {
                    match String::from_utf8(key.to_vec()) {
                        Ok(key) => write_out!(format!("GET {}\n", key).as_bytes()),
                        Err(_) => write_out!(b"Non-utf8 key"),
                    }
                },
                Ok(Command::Set(key, value)) => {
                    match String::from_utf8(key.to_vec()) {
                        Ok(key) => {
                            match String::from_utf8(value.to_vec()) {
                                Ok(value) => write_out!(format!("SET {} = {}\n", key, value).as_bytes()),
                                Err(_) => write_out!(b"Non-utf8 value"),
                            }
                        },
                        Err(_) => write_out!(b"Non-utf8 key"),
                    }
                },
                Err(err) => write_out!(format!("ERR {:?}\n", err).as_bytes()),
            }
        }
        Ok(Async::NotReady)
    }
}