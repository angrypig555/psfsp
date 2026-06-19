// lib.rs - psfsp
// defines the protocol bytes

/// Layout:
/// 0x11
/// Greets the server
pub const GREET: u8 = 0x11;
/// Layout:
/// 0x99 IP
/// Redirection from the server to an ephemeral port, contains the IP address as a string
pub const REDIRECTED: u8 = 0x99;
/// Layout:
/// 0x01 data chunk_num
/// Sends a 1 megabyte chunk of data, alongside the number of the chunk
pub const DATA_CHUNK: u8 = 0x01;
/// Layout:
/// 0x05 hash chunk_num
/// Sends the hash for the chunk number
pub const HASH_CHUNK: u8 = 0x05;
/// Layout:
/// 0x02 len_of_msg number_of_chunks full_hash
/// Sends the number of chunks and the hash of the file
pub const FILE_INFO: u8 = 0x02;
/// Layout:
/// 0x03 file_name
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