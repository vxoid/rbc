use crate::encode::*;
use crate::wallet::*;
use crate::warning;
use crypto::ed25519;
use std::io::Read;
use std::mem;
use std::fs;

pub type Val = u32;

pub struct Transaction {
    value: Val,
    sender: PKey,
    receiver: PKey,
    signature: Sign,
    is_coinbase: bool
}

impl Clone for Transaction {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            sender: self.sender.clone(),
            receiver: self.receiver.clone(),
            signature: self.signature.clone(),
            is_coinbase: self.is_coinbase.clone()
        }
    }
}

#[repr(C)]
struct TransactionSignBytes {
    value: Val,
    sender: PKey,
    receiver: PKey,
    is_coinbase: bool
}

#[repr(C)]
struct TransactionBytes {
    value: Val,
    sender: PKey,
    receiver: PKey,
    signature: Sign,
    is_coinbase: bool,
}

impl Into<Vec<u8>> for Transaction {
    fn into(self) -> Vec<u8> {
        encode(&TransactionBytes {
            value: self.value,
            sender: self.sender,
            receiver: self.receiver,
            signature: self.signature,
            is_coinbase: self.is_coinbase,
        })
    }
}

impl ToString for Transaction {
    fn to_string(&self) -> String {
        if !self.is_coinbase {
            format!(
                "{} -> {} {}🪙",
                hex(self.get_sender()),
                hex(self.get_receiver()),
                self.get_value()
            )
        } else {
            format!(
                "{} mines {}🪙",
                hex(self.get_receiver()),
                self.get_value()
            )
        }
    }
}

impl Transaction {
    pub fn from_file(file: &mut fs::File, count: usize) -> Vec<Self> {
        let mut transactions: Vec<Self> = Vec::with_capacity(count);

        let mut batch: [u8; mem::size_of::<TransactionBytes>()] = [0; mem::size_of::<TransactionBytes>()];
        for _ in 0..count {
            let bytes_read: usize = match file.read(&mut batch) {
                Ok(bytes_read) => bytes_read,
                Err(err) => {
                    warning!(format!("can\'t read file due to {}", err));
                    return Vec::new()
                },
            };

            if bytes_read != mem::size_of::<TransactionBytes>() {
                warning!(format!(
                    "the file is not of expected size, expected {} bytes got {} bytes",
                    mem::size_of::<TransactionBytes>(),
                    bytes_read
                ));
                return Vec::new()
            }

            let transaction: &TransactionBytes = unsafe {
                &*(batch.as_ptr() as *const TransactionBytes)
            };

            transactions.push(Transaction {
                value: transaction.value,
                sender: transaction.sender,
                receiver: transaction.receiver,
                signature: transaction.signature,
                is_coinbase: transaction.is_coinbase,
            })
        }

        transactions
    }

    pub fn new_coinbase(receiver: &Wallet, value: Val) -> Self {
        Self { value, sender: [0; PKEY_SIZE], receiver: receiver.get_pk().clone(), signature: [0; SIGN_SIZE], is_coinbase: true }
    }
    
    pub fn new(sender: &Wallet, receiver: &Wallet, value: Val) -> Self {
        let mut transaction = Self {
            value,
            sender: sender.get_pk().clone(),
            receiver: receiver.get_pk().clone(),
            signature: [0; SIGN_SIZE],
            is_coinbase: false
        };

        transaction.sign(sender);

        transaction
    }

    pub fn get_sender(&self) -> &PKey {
        &self.sender
    }

    pub fn get_receiver(&self) -> &PKey {
        &self.receiver
    }

    pub fn is_receiver(&self, wallet: &Wallet) -> bool {
        self.receiver == wallet.get_pk().clone()
    }

    pub fn is_sender(&self, wallet: &Wallet) -> bool {
        self.sender == wallet.get_pk().clone()
    }

    pub fn get_value(&self) -> &Val {
        &self.value
    }

    pub fn is_valid(&self) -> bool {
        if self.is_coinbase {
            return true;
        }

        ed25519::verify(&self.sign_message(), &self.sender, &self.signature)
    }

    pub fn is_coinbase(&self) -> &bool {
        &self.is_coinbase
    }

    fn sign_message(&self) -> Vec<u8> {
        let bytes = encode(&TransactionSignBytes {
            value: self.value,
            sender: self.sender,
            receiver: self.receiver,
            is_coinbase: self.is_coinbase,
        });

        println!("{}", hex(&bytes));
        sha256_hash(&bytes).to_vec()
    }

    fn sign(&mut self, sender: &Wallet) {
        self.signature = ed25519::signature(&self.sign_message(), sender.get_sk());
    }
}