use std::fs::{File, Metadata};
use std::io::{Error, prelude::*};
use std::net::{TcpListener, TcpStream};
use std::{env, fs};
use std::path::PathBuf;
use std::path::Path;
use std::collections::HashSet;
use std::thread;
use std::sync::Arc;

use psfsp::{ACK, BYE, DATA_CHUNK, FILE_INFO, GET, GREET, NOTEXIST, REDIRECTED};

fn handle_client(mut first_stream: TcpStream, available_files: Arc<HashSet<String>>, shared_directory: Arc<PathBuf>) -> std::io::Result<()> {
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
        return  Err(Error::new(std::io::ErrorKind::PermissionDenied, "client attempted to connect that had a seperate ip from the initial one"));
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
        println!("client requested {}", &filename);
        if !available_files.contains(filename.as_ref()) {
            stream.write_all(&[NOTEXIST])?;
            return Err(Error::new(std::io::ErrorKind::InvalidFilename, "client requested a file that does not exist"));
        }
        let mut requested_file = (*shared_directory).clone();
        requested_file.push(PathBuf::from(filename.to_string()));
        println!("sending client {}", requested_file.display());
        let metadata = fs::metadata(&requested_file)?;
        let file_len = metadata.len();
        let chunks = file_len;
        let chunk_bytes: [u8; 8] = chunks.to_be_bytes();
        let mut f_info_packet: Vec<u8> = Vec::new();
        f_info_packet.push(FILE_INFO);
        f_info_packet.extend(chunk_bytes);
        stream.write_all(&f_info_packet)?;
        stream.read(&mut buffer)?;
        if buffer[0] != ACK {
            return Err(Error::new(std::io::ErrorKind::ConnectionAborted, "Client denied download"));
        }
        let mut file = File::open(requested_file)?;
        let mut f_buffer = vec![0u8; 1_048_576];
        loop {
            let bytes_read = file.read(&mut f_buffer)?;
            if bytes_read == 0 {
                break
            }
            let current_chunk = &f_buffer[..bytes_read];
            stream.write_all(current_chunk)?;
            stream.read(&mut buffer)?;
        }
        stream.write_all(&[BYE])?;
    } else {
       return Err(Error::new(std::io::ErrorKind::InvalidData, "Invalid command"));
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("no arguments were provided\nusage:\nserver.exe [SHARED_DIRECTORY]");
    }
    let shared_dir = PathBuf::from(&args[1]);
    let shared_dir_name_len = args[1].len() + 1;
    println!("server");
    println!("shared directory {:#?}", &shared_dir);
    println!("sharing everything inside of directory");
    println!("contents:");
    let shared_dir_arc = Arc::new(shared_dir);
    let mut files: HashSet<String> = HashSet::new();
    let paths = fs::read_dir(&*shared_dir_arc).unwrap();
    for path in paths {
        let mut cur_file = path.unwrap().path().display().to_string();
        cur_file.drain(0..shared_dir_name_len);
        println!("{}", cur_file);
        files.insert(cur_file.to_string());
    }
    let available_files = Arc::new(files);
    
    println!("files indexed, opening server on port 6434");
    let listener = TcpListener::bind("0.0.0.0:6434")?;
    for stream in listener.incoming() {
        println!("got connection");
        let available_files_clone = Arc::clone(&available_files);
        let shared_dir_clone = Arc::clone(&shared_dir_arc);
        thread::spawn(|| {
            let mut stream = match stream {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Client failed to connect");
                    return;
                }
            };
            let mut buffer = [0; 128];
            let greeting = match stream.read(&mut buffer) {
                Ok(bytes) => bytes,
                Err(e) => {
                    eprintln!("{}", e);
                    return;
                }
            };
            if greeting <= 0 {
                println!("client sent no data");
                return;
            }
            let greet_byte = buffer[0];
            if greet_byte != GREET {
                println!("client sent incorrect magic byte");
                return;
            }
            match handle_client(stream, available_files_clone, shared_dir_clone) {
                Ok(_) => println!("client has been served"),
                Err(e) => eprintln!("{}", e),
            }
        });
        
    }
    Ok(())
}