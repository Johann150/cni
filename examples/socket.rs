use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

/// a TCP server that spews out `a = x` where x is counting up all u32's
/// (total transfered would be ~58GiB)
fn server() {
    let listener = TcpListener::bind("127.0.0.1:42069").expect("port in use");
    eprintln!("server started");

    // accept one connection, then terminate
    let (mut client, _) = listener.accept().unwrap();
    for i in 0..=std::u32::MAX {
        writeln!(client, "a = {}", i).unwrap();
        client.flush().unwrap();

        println!("server at {}", i);
        // sleep so the client has a chance to keep up with
        // the server and hopefully gets scheduled
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

fn main() {
    // start the server
    std::thread::spawn(server);

    // allow some time for the server to be set up
    std::thread::sleep(std::time::Duration::from_secs(1));

    eprintln!("starting read");
    let stream = TcpStream::connect("127.0.0.1:42069").unwrap();

    let parser = cni_format::CniParser::new(
        utf::decode_utf8(stream.bytes().filter_map(Result::ok)).filter_map(Result::ok),
    );

    for (key, value) in parser.filter_map(Result::ok) {
        println!("client at {}", value);
    }
}
