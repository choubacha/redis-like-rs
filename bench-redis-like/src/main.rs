use std::net::TcpStream;
use std::io::{self, Write, Read};
use std::thread;
use std::thread::JoinHandle;

fn run_bench(n: u32) {
    let mut stream = TcpStream::connect("localhost:9000").unwrap();

    for _ in 0..n {
        let _ = stream.write(b"get hello\n");
        //let mut total_read = 0;
        //let mut response: Vec<u8> = Vec::with_capacity(1024);
        let mut buf = [0; 1024];
        loop {
            match stream.read(&mut buf) {
                Ok(0) => {
                    break;
                },
                Ok(n) => {
                    //total_read += n;
                    if buf[n - 1] == 10 { // new line
                        //response.extend_from_slice(&buf[0..n - 1]);
                        break;
                    } else {
                        //response.extend_from_slice(&buf[0..n]);
                        continue;
                    }
                },
                Err(e) => panic!("something bad: {}", e),
            }
        }
    }
}

fn main() {
    let handles: Vec<JoinHandle<()>> = (0..10)
        .map(|_| thread::spawn(|| run_bench(100_000)))
        .collect();
    handles.into_iter().for_each(|h| { h.join().unwrap(); });
}
