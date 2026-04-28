use std::{io::Read, net::TcpStream};

fn handle_stream(mut stream: TcpStream) {
    println!("Success!");
    let mut buffer: [u8; 10] = [0; 10];
    let i = stream.read(&mut buffer);

    for e in buffer {
        println!("{}", e);
    }

    println!("{:?}", i);
}

fn main() -> std::io::Result<()> {
    println!("Hello, world!");

    let stream = TcpStream::connect("0.0.0.0:1234")?;

    handle_stream(stream);

    Ok(())
}
