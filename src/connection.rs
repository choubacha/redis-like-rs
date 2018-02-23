use futures::{Async, Future, Poll, Stream};
use tokio_io::io::WriteHalf;
use tokio_io::AsyncRead;
use tokio::net::TcpStream;
use command::CommandStream;
use futures::sync::mpsc;
use std::io;
use std::io::Write;
use db::{DbResult, Transaction};

pub struct Connection {
    writer: WriteHalf<TcpStream>,
    commands: CommandStream,
    db_channel: mpsc::UnboundedSender<Transaction>,
    db_result_sender: mpsc::UnboundedSender<DbResult>,
    db_result_receiver: mpsc::UnboundedReceiver<DbResult>,
}

impl Connection {
    pub fn new(socket: TcpStream, db_channel: mpsc::UnboundedSender<Transaction>) -> Self {
        let (reader, writer) = socket.split();
        let commands = CommandStream::new(reader);
        let (db_result_sender, db_result_receiver) = mpsc::unbounded();
        Connection {
            db_channel,
            db_result_sender,
            db_result_receiver,
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

        if let Async::Ready(Some(result)) = self.db_result_receiver.poll().unwrap() {
            match result {
                DbResult::Written => write_out!(b"1\n"),
                DbResult::NotFound => write_out!(b"\n"),
                DbResult::Found(bytes) => {
                    write_out!(&bytes);
                    write_out!(b"\n");
                }
                DbResult::NotWritten => write_out!(b"0\n"),
            }
        }

        if let Async::Ready(command) = self.commands.poll()? {
            match command {
                Some(Ok(cmd)) => {
                    let txn = Transaction::new(cmd, self.db_result_sender.clone());
                    self.db_channel.unbounded_send(txn.clone()).unwrap();
                }
                Some(Err(err)) => write_out!(format!("ERR {:?}\n", err).as_bytes()),
                None => return Ok(Async::Ready(())),
            };
        }
        Ok(Async::NotReady)
    }
}
