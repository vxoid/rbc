use crate::wallet::Wallet;
use crate::transaction::*;
use crate::encode::*;
use crate::verbose;
use std::mem;
use std::io;

pub const BLOCK_STATIC_SIZE: usize = mem::size_of::<BlockStatic>();
pub const MINING_REWARD: Val = 50;

#[repr(C)]
struct BlockHashBytes {
    nonce: u128,
    height: usize,
    timestamp: u128,
    prev_hash: Hash,
    transactions: usize
}

pub struct BlockStatic {
    prev_hash: Hash,
    timestamp: u128,
    hash: Hash,
    height: usize,
    nonce: u128,
}

impl Clone for BlockStatic {
    fn clone(&self) -> Self {
        Self {
            prev_hash: self.prev_hash.clone(),
            timestamp: self.timestamp.clone(),
            height: self.height.clone(),
            nonce: self.nonce.clone(),
            hash: self.hash.clone()
        }
    }
}

pub struct Block {
    block: BlockStatic,
    transactions: Vec<Transaction>
}

impl Clone for Block {
    fn clone(&self) -> Self {
        Self {
            transactions: self.transactions.clone(),
            block: self.block.clone()
        }
    }
}

impl Into<Vec<u8>> for Block {
    fn into(self) -> Vec<u8> {
        let transactios_count: usize = self.transactions.len();
        let mut bytes: Vec<u8> = encode(&(self.block, transactios_count));

        for transaction in self.transactions {
            let mut transaction_bytes: Vec<u8> = transaction.into();
            
            bytes.append(&mut transaction_bytes)
        }

        bytes
    }
}

impl From<(BlockStatic, Vec<Transaction>)> for Block {
    fn from(value: (BlockStatic, Vec<Transaction>)) -> Self {
        Self { block: value.0, transactions: value.1 }
    }
}

impl Block {
    pub fn new_genesis(height: usize) -> io::Result<Self> {
        Block::new_with_tx(&[],  [0; SHA256_HASH_SIZE], height)
    }

    pub fn new(miner_address: &Wallet, transactions: &[Transaction], prev_hash: Hash, height: usize) -> io::Result<Self> {
        let reward: Transaction = Transaction::new_coinbase(miner_address, MINING_REWARD);
        let mut transactions: Vec<Transaction> = transactions.to_vec();
        
        transactions.insert(0, reward);

        Self::new_with_tx(&transactions, prev_hash, height)
    }

    fn new_with_tx(transactions: &[Transaction], prev_hash: Hash, height: usize) -> io::Result<Self> {
        let timestamp: u128 = get_unix_time()
            .map_err(|err| io::Error::new(io::ErrorKind::OutOfMemory, err))?;

        let mut block: Self = Self {
            transactions: transactions.to_vec(),
            block: BlockStatic {
                prev_hash,
                timestamp,
                hash: [0; SHA256_HASH_SIZE],
                height,
                nonce: 0
            }
        };

        block.pow();

        Ok(block)
    }

    pub fn has_valid_transactions(&self, verbose: bool) -> bool {
        let coinbase = match self.transactions.get(0) {
            Some(coinbase) => coinbase,
            None => return true,
        };
        if !coinbase.is_coinbase().clone() {
            if verbose {
                verbose!("first transaction in the block must be a coinbase")
            }

            return false;
        }
        if coinbase.get_value() != &MINING_REWARD {
            if verbose {
                verbose!("the coinbase gives more or less than coinbase")
            }
            return false;
        }

        for transaction in &self.transactions[1..] {
            if transaction.is_coinbase().clone() {
                if verbose {
                    verbose!(format!("{} ({}), there can be only 1 coinbase in the block", transaction.to_string(), ut_to_str(self.block.timestamp)))
                }

                return false;
            }

            if !transaction.is_valid() {
                if verbose {
                    verbose!(format!("{} ({}) is invalid", transaction.to_string(), ut_to_str(self.block.timestamp)))
                }

                return false;
            }
        }
        
        true
    }

    pub fn get_hash(&self) -> &Hash {
        &self.block.hash
    }

    pub fn get_prev_hash(&self) -> &Hash {
        &self.block.prev_hash
    }
    
    pub fn get_timestamp(&self) -> &u128 {
        &self.block.timestamp   
    }

    pub fn get_transactions(&self) -> &[Transaction] {
        &self.transactions
    }

    pub fn create_hash(&self) -> Hash {
        let data: Vec<u8> = self.hash_data();

        sha256_hash(&data)
    }
    
    fn hash_data(&self) -> Vec<u8> {
        let block: Self = self.clone();
        let transactios_count: usize = block.transactions.len();

        let mut bytes: Vec<u8> = encode(&BlockHashBytes {
            nonce: block.block.nonce,
            height: block.block.height,
            transactions: transactios_count,
            timestamp: block.block.timestamp,
            prev_hash: block.block.prev_hash,
        });

        for transaction in block.transactions {
            let mut transaction_bytes: Vec<u8> = transaction.into();
            
            bytes.append(&mut transaction_bytes)
        }

        bytes
    }

    fn pow(&mut self) {
        loop {
            if let Some(valid_hash) = self.validate() {
                self.block.hash = valid_hash;
                break;
            }
            self.block.nonce += 1;
        }
    }

    fn validate(&self) -> Option<Hash> {
        let hash: Hash = self.create_hash();

        let mut zeroes: String = String::new();
        for _ in 0..self.block.height {
            zeroes.push('0')
        }

        let hex: String = hex(&hash);
        
        if &hex[..self.block.height as usize] == &zeroes {
            return Some(hash);
        }

        None
    }
}