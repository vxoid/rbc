use std::io;
use crypto::ed25519;
use crate::encode::*;

pub const PKEY_SIZE: usize = 32;
pub type PKey = [u8; PKEY_SIZE];
pub const SIGN_SIZE: usize = 64;
pub type Sign = [u8; SIGN_SIZE];
pub type SKey = [u8; 64];

pub struct Wallet {
    pk: PKey, // public key
    sk: SKey // secret key
}

impl Wallet {
    pub fn new() -> io::Result<Self> {
        let unix_time: u128 = get_unix_time()
            .map_err(|err| io::Error::new(io::ErrorKind::OutOfMemory, err))?;
        let seed: Hash = sha256_hash(&encode(&unix_time));

        let (sk, pk) = ed25519::keypair(&seed);

        Ok(Self { pk, sk })
    }

    pub fn get_pk(&self) -> &PKey {
        &self.pk
    }

    pub fn get_sk(&self) -> &SKey {
        &self.sk
    }
}