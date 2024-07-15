use std::{io::Write, net::TcpStream, thread, time::Duration};

use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};

fn main() {
    let mut connector = SslConnector::builder(SslMethod::tls()).unwrap();
    connector.set_verify(SslVerifyMode::NONE);
    let connector = connector.build();

    let stream = TcpStream::connect("127.0.0.1:1337").unwrap();
    let mut stream = connector.connect("127.0.0.1", stream).unwrap();

    stream.write_all("CONNECT\nDefender94\npass\t".as_bytes()).unwrap();

    stream.flush().unwrap();
    loop {
        stream.write_all("SEND\n\nSOME MESSAGE\t".as_bytes()).unwrap();
        stream.flush().unwrap();
        thread::sleep(Duration::from_secs(1));
    }
}
