use std::fs::{File, Metadata};
use std::io::{BufReader, Error, prelude::*};
use std::net::{TcpListener, TcpStream};
use std::{env, fs};
use std::path::PathBuf;
use std::path::Path;
use std::collections::HashSet;
use std::thread;
use std::sync::Arc;
use std::net::Shutdown;

use rcgen::generate_simple_self_signed;
use rustls::{ServerConfig, ServerConnection, Stream};
use rustls_pki_types::{CertificateDer, PrivateKeyDer};

use psfsp::{ACK, AUTH, AVAILABLE, BYE, DATA_CHUNK, FAIL, FILE_INFO, FILE_METADATA, GET, GREET, HASH, NOTEXIST, REDIRECTED, hash};

fn cert_handler() -> ServerConfig {
    let home_dir = std::env::var("USERPROFILE") 
        .or_else(|_| std::env::var("HOME"))     
        .expect("could not find home dir");
    let mut cert_dir = PathBuf::from(home_dir);
    cert_dir.push(".psfsp");
    fs::create_dir_all(&cert_dir).expect("failed to create directory for keys");
    let cert_path = cert_dir.join("server.crt");
    let key_path = cert_dir.join("server.key");

    if !cert_path.exists() || !key_path.exists() {
        let subject_alt_names = vec!["psfsp".to_string()];
        let certified_key = generate_simple_self_signed(subject_alt_names).expect("failed to generate TLS certificates");

        File::create(&cert_path).unwrap().write_all(certified_key.cert.pem().as_bytes()).unwrap();
        File::create(&key_path).unwrap().write_all(certified_key.signing_key.serialize_pem().as_bytes()).unwrap();
    }

    let certs = rustls_pemfile::certs(&mut BufReader::new(File::open(cert_path).unwrap()))
        .collect::<Result<Vec<_>, _>>().unwrap();
    let key = rustls_pemfile::private_key(&mut BufReader::new(File::open(key_path).unwrap()))
        .unwrap().unwrap();

    ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .expect("failed to build ServerConfig")
}

fn handle_client(mut first_stream: TcpStream, available_files: Arc<HashSet<String>>, shared_directory: Arc<PathBuf>, username: Arc<String>, password: Arc<String>, tls_conf: Arc<ServerConfig>) -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:0")?;
    let new_port = listener.local_addr()?.port();
    println!("got new temp port {}", new_port);
    let new_port_u8 = new_port.to_be_bytes();
    let redirection = [REDIRECTED, new_port_u8[0], new_port_u8[1]];
    let client_ip = first_stream.peer_addr()?.ip();
    first_stream.write_all(&redirection)?;
    drop(first_stream);
    let (mut rawstream, client_addr) = listener.accept()?;
    let mut server_conn = ServerConnection::new(tls_conf).unwrap();
    let mut stream = Stream::new(&mut server_conn, &mut rawstream);
    let new_client_ip = client_addr.ip();
    if new_client_ip != client_ip {
        return  Err(Error::new(std::io::ErrorKind::PermissionDenied, "client attempted to connect that had a seperate ip from the initial one"));
    }   
    println!("client succesfully verified on new port");
    let mut buffer = [0; 128];
    stream.write_all(&[AUTH])?;
    stream.read(&mut buffer)?;
    let auth_first_byte = buffer[0];
    if auth_first_byte != AUTH {
        return Err(Error::new(std::io::ErrorKind::InvalidInput, "Client did not send an AUTH packet back"));
    }
    let username_len = buffer[1] as usize;
    let username_end = username_len + 2;
    if username_end + 1 > buffer.len() {
        return Err(Error::new(std::io::ErrorKind::InvalidData, "Malformed packet"));
    }
    let pass_len_index = username_end + 8;
    let c_username = String::from_utf8_lossy(&buffer[2..username_end]);
    let password_len = buffer[username_end] as usize;
    let password_start = username_end + 1;
    let password_end = password_len + password_start;
    if password_end > buffer.len() {
        return Err(Error::new(std::io::ErrorKind::InvalidData, "Malformed packet"));
    }
    let c_password = String::from_utf8_lossy(&buffer[password_start..password_end]);
    if c_username != *username || c_password != *password {
        println!("Client tried to auth with incorrect credentials\nUsername: {}\nPassword: {}", c_username, c_password);
        stream.write_all(&[FAIL])?;
        rawstream.shutdown(Shutdown::Write)?;
        return Err(Error::new(std::io::ErrorKind::PermissionDenied, "client tried to auth with incorrect credentials"));
    }
    stream.write_all(&[ACK])?;
    println!("authenticated succesfully");
    loop {
        stream.read(&mut buffer)?;
        let first_byte = buffer[0];
        if first_byte == GET {
            let filename_len = buffer[1] as usize;
            let start_index = 2;
            let end_index = start_index + filename_len;
            let filename_raw = &buffer[start_index..end_index];
            let filename = String::from_utf8_lossy(&filename_raw);
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
            let mut file = File::open(&requested_file)?;
            let mut f_buffer = vec![0u8; 51200];
            loop {
                let bytes_read = file.read(&mut f_buffer)?;
                if bytes_read == 0 {
                    break
                }
                let current_chunk = &f_buffer[..bytes_read];
                stream.write_all(current_chunk)?;
                stream.read(&mut buffer)?;
                if buffer[0] != ACK {
                    return Err(Error::new(std::io::ErrorKind::ConnectionAborted, "Client aborted download"));
                }
            }
            println!("calculating hash");
            let hash_server = hash(&requested_file);
            let hash_len = hash_server.len().to_be_bytes();
            let mut hash_packet: Vec<u8> = Vec::new();
            hash_packet.push(HASH);
            hash_packet.extend(hash_len);
            hash_packet.extend(hash_server.as_bytes());
            stream.write_all(&hash_packet)?;
        } else if first_byte == AVAILABLE {
            println!("client requested available files");
            let num_of_files = available_files.len() as u64;
            let mut ack_packet = Vec::new();
            ack_packet.push(ACK);
            ack_packet.extend_from_slice(&num_of_files.to_be_bytes());
            stream.write_all(&ack_packet)?;
            for file in available_files.iter() {
                let mut file_info = Vec::new();
                file_info.push(FILE_METADATA);
                let mut requested_file = PathBuf::new();
                requested_file.push(&**shared_directory);
                requested_file.push(file);
                let metadata = fs::metadata(&requested_file)?;
                let file_len = metadata.len();
                file_info.extend_from_slice(&file_len.to_be_bytes());
                let filename_len = file.len() as u64;
                file_info.extend_from_slice(&filename_len.to_be_bytes());
                file_info.extend_from_slice(&file.as_bytes());
                stream.write_all(&file_info)?;
                stream.read(&mut buffer)?;
            }

        } else if first_byte == BYE {
            println!("client finished session");
            let _ = stream.flush();
            break
        } else {
            return Err(Error::new(std::io::ErrorKind::InvalidData, "Invalid command"));
        }
    }
    stream.conn.send_close_notify();
    let _ = stream.flush();
    rawstream.shutdown(Shutdown::Write)?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    println!("verifying tls certificates");
    let tls_conf = Arc::new(cert_handler());
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        panic!("no arguments were provided\nusage:\nserver.exe [SHARED_DIRECTORY] [USERNAME] [PASSWORD]");
    }
    let shared_dir = PathBuf::from(&args[1]);
    let shared_dir_name_len = args[1].len() + 1;
    let username = &args[2];
    let password = &args[3];
    let username_arc = Arc::new(String::from(username));
    let password_arc = Arc::new(String::from(password));
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
        let username_clone = Arc::clone(&username_arc);
        let password_clone = Arc::clone(&password_arc);
        let tls_conf_clone = Arc::clone(&tls_conf);
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
            match handle_client(stream, available_files_clone, shared_dir_clone, username_clone, password_clone, tls_conf_clone) {
                Ok(_) => println!("client has been served"),
                Err(e) => eprintln!("{}", e),
            }
        });
        
    }
    Ok(())
}