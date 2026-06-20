use std::io::prelude::*;
use std::net::TcpStream;

use psfsp::GREET;

fn main() -> std::io::Result<()> {
    println!("client");
    let mut stream = TcpStream::connect("127.0.0.1:6434")?;
    stream.write_all(&[GREET])?;
    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;
    let bytes = [buffer[1], buffer[2]];
    let redirected_port = u16::from_be_bytes(bytes);
    println!("new port is {}", redirected_port);
    Ok(())
}