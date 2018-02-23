use bytes::{Bytes, BytesMut};
use tokio_io::io::ReadHalf;
use tokio_io::AsyncRead;
use tokio::net::TcpStream;
use futures::{Async, Poll, Stream};
use std::io;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Get(Bytes),
    Set(Bytes, Bytes),
}

impl Command {
    fn from(mut bytes: Bytes) -> Result<Command, ErrorKind> {
        // Find first space
        if let Some(first_space_index) = bytes.iter().position(|b| b == &b' ') {
            let cmd = bytes.split_to(first_space_index);
            bytes.advance(1);

            match cmd.as_ref() {
                b"get" => {
                    if bytes.contains(&b' ') {
                        Err(ErrorKind::InvalidKey)
                    } else if bytes.len() == 0 {
                        Err(ErrorKind::KeyNotFound)
                    } else {
                        Ok(Command::Get(bytes))
                    }
                }
                b"set" => {
                    if let Some(first_space_index) = bytes.iter().position(|b| b == &b' ') {
                        let key = bytes.split_to(first_space_index);
                        bytes.advance(1);
                        if key.len() == 0 {
                            Err(ErrorKind::KeyNotFound)
                        } else if bytes.len() == 0 {
                            Err(ErrorKind::ValueNotFound)
                        } else {
                            Ok(Command::Set(key, bytes))
                        }
                    } else {
                        Err(ErrorKind::KeyNotFound)
                    }
                }
                _ => Err(ErrorKind::CommandNotFound),
            }
        } else {
            Err(ErrorKind::CommandNotFound)
        }
    }
}

#[derive(Debug)]
pub struct CommandStream {
    read_socket: ReadHalf<TcpStream>,
    rd: BytesMut,
}

impl CommandStream {
    /// Create a new `Lines` codec backed by the socket
    pub fn new(read_socket: ReadHalf<TcpStream>) -> Self {
        CommandStream {
            read_socket,
            rd: BytesMut::new(),
        }
    }

    /// Read data from the socket.
    ///
    /// This only returns `Ready` when the socket has closed.
    fn fill_read_buf(&mut self) -> Poll<(), io::Error> {
        loop {
            // Ensure the read buffer has capacity.
            //
            // This might result in an internal allocation.
            self.rd.reserve(1024);

            // Read data into the buffer.
            let n = try_ready!(self.read_socket.read_buf(&mut self.rd));

            if n == 0 {
                // Nothing read, socket closed
                return Ok(Async::Ready(()));
            }
        }
    }
}

impl Stream for CommandStream {
    type Item = Result<Command, ErrorKind>;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        // First, read any new data that might have been received off the socket
        let sock_closed = self.fill_read_buf()?.is_ready();

        // Now, try finding lines

        if let Some(pos) = self.rd.iter().position(|b| *b == b'\n') {
            // Remove the line from the read buffer and set it to `line`.
            let mut line = self.rd.split_to(pos + 1);

            // Drop the trailing \r\n
            line.split_off(pos);

            // Return the line
            return Ok(Async::Ready(Some(Command::from(line.freeze()))));
        }

        if sock_closed {
            Ok(Async::Ready(None))
        } else {
            Ok(Async::NotReady)
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    CommandNotFound,
    InvalidKey,
    KeyNotFound,
    ValueNotFound,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_parse_to_set() {
        let bytes = Bytes::from(&b"set hello world yessss"[..]);
        assert_eq!(
            Command::from(bytes),
            Ok(Command::Set(
                Bytes::from(&b"hello"[..]),
                Bytes::from(&b"world yessss"[..])
            ))
        );
    }

    #[test]
    fn it_can_parse_to_get() {
        let bytes = Bytes::from(&b"get hello"[..]);
        assert_eq!(
            Command::from(bytes),
            Ok(Command::Get(Bytes::from(&b"hello"[..])))
        );
    }

    #[test]
    fn it_can_detect_no_command() {
        let bytes = Bytes::from(&b"hello world"[..]);
        assert_eq!(Command::from(bytes), Err(ErrorKind::CommandNotFound));
    }

    #[test]
    fn it_can_detect_invalid_key() {
        let bytes = Bytes::from(&b"get invalid key"[..]);
        assert_eq!(Command::from(bytes), Err(ErrorKind::InvalidKey));
    }

    #[test]
    fn it_can_detect_no_key_for_get() {
        let bytes = Bytes::from(&b"get "[..]);
        assert_eq!(Command::from(bytes), Err(ErrorKind::KeyNotFound));
    }

    #[test]
    fn it_can_detect_no_key_for_set() {
        let bytes = Bytes::from(&b"set  "[..]);
        assert_eq!(Command::from(bytes), Err(ErrorKind::KeyNotFound));
        let bytes = Bytes::from(&b"set "[..]);
        assert_eq!(Command::from(bytes), Err(ErrorKind::KeyNotFound));
    }

    #[test]
    fn it_can_detect_no_value_for_set() {
        let bytes = Bytes::from(&b"set hello "[..]);
        assert_eq!(Command::from(bytes), Err(ErrorKind::ValueNotFound));
    }
}
