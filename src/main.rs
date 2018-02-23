extern crate bytes;
#[macro_use]
extern crate futures;
extern crate tokio;
#[macro_use]
extern crate tokio_io;

use futures::{Future, Stream};
use tokio::executor::current_thread;
use tokio::net::TcpListener;
use std::thread;

mod command;
mod connection;
mod db;
use connection::Connection;

fn main() {
    let addr = "0.0.0.0:9000".parse().unwrap();
    let tcp = TcpListener::bind(&addr).unwrap();

    // every connection needs a channel to listen for results. when a command
    // comes in, it queues the command with the receiver and then waits for
    // a response
    let (tx, db) = db::Db::new();

    thread::spawn(move || {
        current_thread::run(|_| {
            current_thread::spawn(db);
        });
    });

    let server = tcp.incoming()
        .for_each(move |cxn| {
            let tx = tx.clone();
            let conn = Connection::new(cxn, tx)
                .map(|_| println!("closed!"))
                .map_err(|e| println!("err: {:?}", e));
            println!("new connection!");
            current_thread::spawn(conn);
            Ok(())
        })
        .map_err(|err| println!("server error: {:?}", err));

    current_thread::run(|_| {
        current_thread::spawn(server);
    });
}
