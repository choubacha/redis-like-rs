use command::Command;
use futures::sync::mpsc;
use futures::{Async, Future, Poll, Stream};
use std::collections::HashMap;
use bytes::Bytes;

#[derive(Debug)]
pub enum DbResult {
    Written,
    #[allow(dead_code)] NotWritten,
    Found(Bytes),
    NotFound,
}

#[derive(Clone)]
pub struct Transaction {
    command: Command,
    db_result_channel: mpsc::UnboundedSender<DbResult>,
}

impl Transaction {
    pub fn new(command: Command, db_result_channel: mpsc::UnboundedSender<DbResult>) -> Self {
        Self {
            command,
            db_result_channel,
        }
    }

    fn send_result<F>(self, f: F)
    where
        F: FnOnce(Command) -> DbResult,
    {
        let _ = self.db_result_channel.unbounded_send(f(self.command));
    }
}

pub struct Db {
    transaction_channel: mpsc::UnboundedReceiver<Transaction>,
    map: HashMap<Bytes, Bytes>,
}

impl Db {
    pub fn new() -> (mpsc::UnboundedSender<Transaction>, Db) {
        let (tx, transaction_channel) = mpsc::unbounded();
        (
            tx,
            Db {
                transaction_channel,
                map: HashMap::new(),
            },
        )
    }
}

impl Future for Db {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<(), ()> {
        while let Async::Ready(Some(txn)) = self.transaction_channel.poll()? {
            txn.send_result(|command| match command {
                Command::Get(key) => match self.map.get(&key) {
                    Some(value) => DbResult::Found(value.clone()),
                    None => DbResult::NotFound,
                },
                Command::Set(key, value) => {
                    self.map.insert(key, value);
                    DbResult::Written
                }
            });
        }
        Ok(Async::NotReady)
    }
}
