use chrono::TimeZone;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use std::time;
use std::mem;

pub const SHA256_HASH_SIZE: usize = 32;
pub type Hash = [u8; SHA256_HASH_SIZE];

pub fn sha256_hash(bytes: &[u8]) -> Hash {
    let mut hasher: Sha256 = Sha256::new();
    hasher.input(bytes);

    let mut hash: [u8; SHA256_HASH_SIZE] = [0; SHA256_HASH_SIZE];
    hasher.result(&mut hash);

    hash
}

pub fn encode<T>(reference: &T) -> Vec<u8> {
    let size: usize = mem::size_of::<T>();

    let ptr: *const u8 = reference as *const T as *const u8;

    let mut bytes: Vec<u8> = Vec::new();
    for i in 0..size {
        let ptr: *const u8 = (ptr as usize + i) as *const u8;

        bytes.push(unsafe { *ptr })
    }

    bytes
}

pub fn hex(bytes: &[u8]) -> String {
    let mut string: String = String::new();
    for byte in bytes {
        string.push_str(&format!("{:02x}", byte))
    }
    string
}

pub fn get_unix_time() -> Result<u128, time::SystemTimeError> {
    let timestamp: u128 = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)?
        .as_nanos();
    Ok(timestamp)
}

pub fn ut_to_str(nanos: u128) -> String {
    let datetime = chrono::Utc.timestamp_nanos(nanos as i64); // UPDATE THIS WHEN THE i64 WONT BE ABLE TO CONTAIN ALL MICROS FROM UNIX EPOCH
    datetime.format("%H:%M UTC %d/%m/%Y").to_string()
}