use futures::{Async, Future, Poll, Stream};
use tokio_io::io::WriteHalf;
use tokio_io::AsyncRead;
use tokio::net::TcpStream;
use command::{Command, CommandStream};
use futures::sync::mpsc;
use std::io;
use std::io::Write;

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
        Connection {
            tx,
            rx,
            commands,
            writer,
        }
    }
}

impl Future for Connection {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {
        macro_rules! write_out {
            ($e:expr) => ({ let _ = try_nb!(self.writer.write($e)); })
        }

        while let Async::Ready(command) = self.commands.poll()? {
            match command {
                Some(Ok(Command::Get(key))) => match String::from_utf8(key.to_vec()) {
                    Ok(key) => write_out!(format!("GET {}\n", key).as_bytes()),
                    Err(_) => write_out!(b"Non-utf8 key"),
                },
                Some(Ok(Command::Set(key, value))) => match String::from_utf8(key.to_vec()) {
                    Ok(key) => match String::from_utf8(value.to_vec()) {
                        Ok(value) => write_out!(format!("SET {} = {}\n", key, value).as_bytes()),
                        Err(_) => write_out!(b"Non-utf8 value"),
                    },
                    Err(_) => write_out!(b"Non-utf8 key"),
                },
                Some(Err(err)) => write_out!(format!("ERR {:?}\n", err).as_bytes()),
                None => return Ok(Async::Ready(())),
            };
        }
        Ok(Async::NotReady)
    }
}
