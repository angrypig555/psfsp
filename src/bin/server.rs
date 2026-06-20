use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

use psfsp::{GET, GREET, REDIRECTED};

fn handle_client(mut first_stream: TcpStream) -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:0")?;
    let new_port = listener.local_addr()?.port();
    println!("got new temp port {}", new_port);
    let new_port_u8 = new_port.to_be_bytes();
    let redirection = [REDIRECTED, new_port_u8[0], new_port_u8[1]];
    let client_ip = first_stream.peer_addr()?.ip();
    first_stream.write_all(&redirection)?;
    drop(first_stream);
    let (mut stream, client_addr) = listener.accept()?;
    let new_client_ip = client_addr.ip();
    if new_client_ip != client_ip {
        println!("client attempted to connect that had a seperate ip from the initial one");
        return  Ok(())
    }
    println!("client succesfully verified on new port");
    let mut buffer = [0; 128];
    stream.read(&mut buffer)?;
    let first_byte = buffer[0];
    let filename_len = buffer[1] as usize;
    let start_index = 2;
    let end_index = start_index + filename_len;
    let filename_raw = &buffer[start_index..end_index];
    let filename = String::from_utf8_lossy(&filename_raw);
    if first_byte == GET {
        println!("client requested {}", filename);
    } 
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
        handle_client(stream)?;
    }
    Ok(())
}