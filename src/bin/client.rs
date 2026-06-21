use core::num;
use std::fmt::format;
use std::fs;
use std::fs::File;
use std::io::Error;
use std::io::prelude::*;
use std::net::TcpStream;
use std::env;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use indicatif::ProgressBar;

use psfsp::AUTH;
use psfsp::AVAILABLE;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls_pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{ClientConfig, ClientConnection, Stream, DigitallySignedStruct};

use psfsp::BYE;
use psfsp::HASH;
use psfsp::hash;
use psfsp::{GET, GREET, NOTEXIST, ACK, FAIL};

#[derive(Debug)]
struct BlindTrustVerifier;

impl ServerCertVerifier for BlindTrustVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer,
        _intermediates: &[CertificateDer],
        _server_name: &ServerName,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ED25519,
            rustls::SignatureScheme::RSA_PSS_SHA256,
        ]
    }
}

fn load_client_tls_config() -> ClientConfig {
    let mut config = ClientConfig::builder()
        .with_root_certificates(rustls::RootCertStore::empty())
        .with_no_client_auth();

    config.dangerous().set_certificate_verifier(Arc::new(BlindTrustVerifier));
    config
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        panic!("no arguments were provided\nusage:\nclient.exe [IP] [DOWNLOAD_DIRECTORY]");
    } else if args.len() > 3 {
        panic!("too many arguments were provided\nusage:\nclient.exe [IP] [DOWNLOAD_DIRECTORY]");
    }
    let stdin = io::stdin();
    let ip = &args[1];
    let download_directory = &args[2];
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
    let mut rawstream = TcpStream::connect(format!("{}:{}", ip, redirected_port))?;
    let tls_config = Arc::new(load_client_tls_config());
    let dummy_name = ServerName::try_from("psfsp.internal").unwrap().to_owned();
    let mut client_conn = ClientConnection::new(tls_config, dummy_name)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let mut stream = Stream::new(&mut client_conn, &mut rawstream);
    stream.read(&mut buffer)?;
    if buffer[0] == AUTH {
        print!("Authentication required\nUsername: ");
        io::stdout().flush()?;
        let mut username = String::new();
        io::stdin().read_line(&mut username).unwrap();
        print!("Password: ");
        io::stdout().flush()?;
        let mut password = String::new();
        io::stdin().read_line(&mut password).unwrap();
        let mut auth_packet = Vec::new();
        auth_packet.push(AUTH);
        let username_len = username.trim().len();
        let password_len = password.trim().len();
        auth_packet.push(username_len.try_into().unwrap());
        auth_packet.extend(username.trim().as_bytes());
        auth_packet.push(password_len.try_into().unwrap());
        auth_packet.extend(password.trim().as_bytes());
        println!("please wait...");
        stream.write_all(&auth_packet)?;
        stream.read(&mut buffer)?;
        if buffer[0] == ACK {
            println!("succesfully authenticated");
        } else {
            return Err(Error::new(std::io::ErrorKind::PermissionDenied, "Authentication failed"))
        }
        
    }
    println!("psfsp example client");
    loop {
        println!("please select what you would like to do");
        print!("[1] query available files\n[2] download a file\n[3] exit\n");
        io::stdout().flush()?;
        let mut option_raw = String::new();
        io::stdin().read_line(&mut option_raw).unwrap();
        let option = option_raw.trim();
        match option {
            "1" => {
                stream.write_all(&[AVAILABLE])?;
                stream.read(&mut buffer)?;
                let mut num_of_files = u64::from_be_bytes(buffer[1..9].try_into().unwrap());
                println!("number of files: {}", num_of_files);
                loop {
                    if num_of_files == 0 {
                        break
                    }
                    let bytes_read = stream.read(&mut buffer)?;
                    if bytes_read == 0 {
                        println!("conenction lost");
                        break;
                    }
                    let filesize_r = u64::from_be_bytes(buffer[1..9].try_into().unwrap());
                    let filename_len = u64::from_be_bytes(buffer[9..17].try_into().unwrap()) as usize;
                    let start_index = 17;
                    let end_index = start_index + filename_len;
                    if end_index > bytes_read {
                        println!("malformed packet received");
                        break;
                    }
                    let filename = String::from_utf8_lossy(&buffer[start_index..end_index]);
                    let mut file_size = filesize_r as f64;
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
                    println!("{} - {:.2}{}", filename, file_size, prefix);
                    num_of_files -= 1;
                    stream.write_all(&[ACK])?;
                }
            },
            "2" => {
                print!("enter filename: ");
                io::stdout().flush()?;
                let mut filename_raw = String::new();
                io::stdin().read_line(&mut filename_raw).unwrap();
                let filename = filename_raw.trim();
                let mut get_packet = Vec::new();
                get_packet.push(GET);
                let filename_len = filename.len();
                get_packet.push(filename_len as u8);
                get_packet.extend_from_slice(filename.as_bytes());
                stream.write_all(&get_packet)?;
                stream.read(&mut buffer)?;
                let response = buffer[0];
                if response == NOTEXIST {
                    println!("requested file was not found on the server");
                    continue;
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
                let pb = ProgressBar::new(file_size_r);
                let mut file_buffer_d = [0u8; 51200];
                let mut bytes_received = 0;
                while bytes_received < file_size_r {
                    let remaining = file_size_r - bytes_received;
                    let max_to_read = std::cmp::min(file_buffer_d.len() as u64, remaining) as usize;
                    let bytes_read = stream.read(&mut file_buffer_d[..max_to_read])?;
                    if bytes_read == 0 {
                        return Err(Error::new(std::io::ErrorKind::UnexpectedEof, "Server dropped the connection"));
                    }
                    file.write_all(&file_buffer_d[..bytes_read])?;
                    bytes_received += bytes_read as u64;
                    pb.set_position(bytes_received);
                    stream.write_all(&[ACK])?;
                }
                pb.finish_with_message("complete");
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
            }
            "3" => {
                println!("goodbye");
                stream.write_all(&[BYE])?;
                break Ok(());
            }
            _ => continue
        }
        
    }
}