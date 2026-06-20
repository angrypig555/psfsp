use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

use psfsp::{GREET, REDIRECTED};

fn handle_client(first_stream: &mut TcpStream) -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:0")?;
    let new_port = listener.local_addr()?.port();
    println!("got new temp port {}", new_port);
    let new_port_u8 = new_port.to_be_bytes();
    let redirection = [REDIRECTED, new_port_u8[0], new_port_u8[1]];
    first_stream.write_all(&redirection)?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    println!("server");
    let listener = TcpListener::bind("0.0.0.0:6434")?;
    for stream in listener.incoming() {
        println!("got connection");
        let mut stream = stream?;
        let mut buffer = [0; 128];
        let greeting = stream.read(&mut buffer)?;
        if greeting <= 0 {
            println!("client sent no data");
            continue;
        }
        let greet_byte = buffer[0];
        if greet_byte != GREET {
            println!("client sent incorrect magic byte");
            continue;
        }
        handle_client(&mut stream)?;
    }
    Ok(())
}