extern crate futures;
extern crate tokio;
extern crate tokio_io;

use futures::{Future, Stream};
use tokio::executor::current_thread;
use tokio::net::TcpListener;
use tokio_io::{io, AsyncRead};

use std::thread;
use std::sync::mpsc::channel;

fn main() {
    let addr = "0.0.0.0:9000".parse().unwrap();
    let tcp = TcpListener::bind(&addr).unwrap();

    let (tx, rx) = channel::<Option<String>>();

    let db = thread::spawn(move || {
        while let Ok(Some(msg)) = rx.recv() {
            println!("Got a message: {}", msg);
        }
    });

    let server = tcp.incoming().for_each(|cxn| {
        let (reader, writer) = cxn.split();

        let conn = io::read_to_end(reader, Vec::new())
            .map(|buf| println!("buf: {:?}", buf))
            .map_err(|err| println!("IO error: {:?}", err));

        current_thread::spawn(conn);

        Ok(())
    })
    .map_err(|err| println!("server error: {:?}", err));

    current_thread::run(|_| {
        current_thread::spawn(server);
    });
}
