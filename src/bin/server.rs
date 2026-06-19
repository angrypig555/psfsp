use std::io::prelude::*;
use std::net::TcpListener;

fn main() -> std::io::Result<()> {
    println!("server");
    let listener = TcpListener::bind("0.0.0.0:6434")?;
    Ok(())
}