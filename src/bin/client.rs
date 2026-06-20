use std::fmt::format;
use std::io::prelude::*;
use std::net::TcpStream;
use std::env;

use psfsp::{GREET, GET};

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        panic!("no arguments were provided\nusage:\nclient.exe [IP] [FILENAME]");
    }
    let ip = &args[1];
    let filename = &args[2];
    println!("client");
    let full_ip = format!("{}:6434", ip);
    let mut stream = TcpStream::connect(full_ip)?;
    stream.write_all(&[GREET])?;
    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;
    let bytes = [buffer[1], buffer[2]];
    let redirected_port = u16::from_be_bytes(bytes);
    println!("new port is {}", redirected_port);
    let mut stream = TcpStream::connect(format!("{}:{}", ip, redirected_port))?;
    let mut get_packet = Vec::new();
    get_packet.push(GET);
    let filename_len = filename.len();
    get_packet.push(filename_len as u8);
    get_packet.extend_from_slice(filename.as_bytes());
    stream.write_all(&get_packet)?;
    Ok(())
}