use std::fmt::format;
use std::fs;
use std::fs::File;
use std::io::Error;
use std::io::prelude::*;
use std::net::TcpStream;
use std::env;
use std::io;
use std::path::PathBuf;

use psfsp::BYE;
use psfsp::HASH;
use psfsp::hash;
use psfsp::{GET, GREET, NOTEXIST, ACK, FAIL};

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        panic!("no arguments were provided\nusage:\nclient.exe [IP] [FILENAME] [DOWNLOAD_DIRECTORY]");
    }
    let stdin = io::stdin();
    let ip = &args[1];
    let filename = &args[2];
    let download_directory = &args[3];
    let download_dir = PathBuf::from(download_directory);
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
    stream.read(&mut buffer)?;
    let response = buffer[0];
    if response == NOTEXIST {
        panic!("Requested file was not found on server");
    }
    let file_size_r  = u64::from_be_bytes(buffer[1..9].try_into().unwrap());
    let mut file_size = file_size_r as f64;
    let mut prefix;
    if file_size > 1099511627776.0 {
        prefix = "TB";
        file_size /= 109951162776.0;
    } else if file_size > 1073741824.0 {
        prefix = "GB";
        file_size /= 1073741824.0;
    } else if file_size > 1048576.0 {
        prefix = "MB";
        file_size /= 1048576.0;
    } else if file_size > 1024.0 {
        prefix = "KB";
        file_size /= 1024.0;
    } else {
        prefix = "B";
    }
    
    
    print!("FILENAME: {}\nSIZE: {:.2}{}\nContinue with download? [y/n] ", filename, file_size, prefix);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    if input.trim() != "y" {
        eprintln!("aborting download");
        stream.write_all(&[FAIL])?;
        return Err(Error::new(std::io::ErrorKind::ConnectionAborted, "Download aborted by user"))
    }
    println!("download starting");
    stream.write_all(&[ACK])?;
    let mut downloaded_file = download_dir.clone();
    downloaded_file.push(filename);
    downloaded_file.set_extension(".part");
    let mut downloaded_file_final_name = download_dir.clone();
    downloaded_file_final_name.push(filename);
    let mut file = File::create(&downloaded_file)?;
    loop {
        let bytes_read = stream.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        if buffer[0] == BYE {
            break;
        }
        file.write_all(&buffer[..bytes_read])?;
        stream.write_all(&[ACK])?;
    }
    fs::rename(downloaded_file, &downloaded_file_final_name)?;
    println!("verifying hash");
    let client_hash = hash(&downloaded_file_final_name);
    stream.read(&mut buffer)?;
    if buffer[0] != HASH {
        println!("File was saved but the hash was not compared, procceed at your own risk");
    }
    let buf_len  = u64::from_be_bytes(buffer[1..9].try_into().unwrap()) as usize;
    let end_index = 9 + buf_len;
    let server_hash = &buffer[9..end_index];
    let server_hash_string = String::from_utf8_lossy(server_hash);
    println!("Downloaded file hash: {}\nHash from server: {}", client_hash, server_hash_string);
    if client_hash != server_hash_string {
        panic!("Hashes do not match, do not trust the downloaded file");
    } else {
        println!("Hashes verified")
    }
    println!("Saved file to {}", downloaded_file_final_name.display());
    Ok(())
}