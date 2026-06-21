// lib.rs - psfsp
// defines the protocol bytes

use std::{fs::File, io};
use std::path::Path;

/// Layout:
/// 0x11
/// Greets the server
pub const GREET: u8 = 0x11;
/// Layout:
/// 0x99 IP
/// Redirection from the server to an ephemeral port, contains the IP address as a string
pub const REDIRECTED: u8 = 0x99;
/// Layout:
/// 0x01 data
/// Sends a 1 megabyte chunk of data, alongside the number of the chunk
pub const DATA_CHUNK: u8 = 0x01;
/// Layout:
/// 0x05 hash chunk_num
/// Sends the hash for the chunk number
pub const HASH_CHUNK: u8 = 0x05;
/// Layout:
/// 1 B   8B 
/// 0x02 number_of_chunks len hash
/// Sends the number of chunks and the hash of the file
pub const FILE_INFO: u8 = 0x02;
/// Layout:
/// 0x03 len file_name
/// Asks the server for a file, done on the ephemeral port
/// Server responds with FILE_INFO
/// Client responds with GET again to begin download
pub const GET: u8 = 0x03;
/// Layout:
/// 0xAA chunk_num
/// Sent back after verifying the hash of the received chunk, will continue the download of the next chunk
pub const ACK: u8 = 0xAA;
/// Layout:
/// 0xFF chunk_num
/// Sent back after verifying the hash of the received chunk is incorrect, server will retry sending the chunk 3 times before closing the connection
pub const FAIL: u8 = 0xFF;
/// Layout:
/// 0x0B
/// Sent back after download is finished and the full hash is verified to be correct or when the server closes the connection
pub const BYE: u8 = 0x0B;
/// Layout:
/// 0x15
/// Sent if the server is still compressing the file
pub const WAIT: u8 = 0x15;
/// Layout:
/// 0x44
/// Sent if the requested file does not exist
pub const NOTEXIST: u8 = 0x44;
/// Layout:
/// 0xAB len hash
/// Sent after the download has been completed to verify the hash
pub const HASH: u8 = 0xAB;
/// Layout:
/// Server sends: 0xAD 0x00 (no auth) / 0x01 (user/pass)
/// Client sends: 0xAD len username len password
pub const AUTH: u8 = 0xAD;
/// Layout:
/// 0xAD
/// Server responds with ack and the number of files and after that the available files along with their sizes, with a file metadata packet
pub const AVAILABLE: u8 = 0xA2;
/// Layout:
/// 0x65 SIZE len FILENAME
/// 1 byte - 8 bytes - 8 bytes - len
/// Client responds with ACK
pub const FILE_METADATA: u8 = 0x65;

/// Hashes a file, returns the hash as a string
pub fn hash(path: &Path) -> String {
    let mut file = match File::open(path) {
        Ok(File) => File,
        Err(e) => panic!("{}", e),
    };
    let mut hasher = blake3::Hasher::new();
    match io::copy(&mut file, &mut hasher) {
        Ok(_) => println!("hashed succesfully"),
        Err(e) => {panic!("{}", e)},
    }
    let hash = hasher.finalize();
    hash.to_hex().to_string()
}