use crate::{transaction::*, verbose};
use crate::wallet::Wallet;
use crate::block::Block;
use crate::db::BlockDB;
use crate::encode::*;
use std::io;

pub const DIFICULTY: u8 = 2;

pub struct Blockchain {
    db: BlockDB
}

impl Blockchain {
    pub fn load(path: &str) -> io::Result<Self> {
        let mut db: BlockDB = BlockDB::open(path)?;
        
        if db.len().clone() < 1 {
            db.push(Block::new_genesis(DIFICULTY as usize)?)?
        } 

        Ok(Self { db })
    }

    pub fn mine(&mut self, miner: &Wallet, transactions: &[Transaction]) -> io::Result<()> {
        let last_block: Block = match self.db.last_block() {
            Some(block) => block,
            None => return Err(io::Error::new(io::ErrorKind::NotFound, "the blockchain is empty, there should be a genesis block, try checking is the db file corrupted")),
        };

        let prev_hash: &Hash = last_block.get_hash();
        let block: Block = Block::new(miner, transactions, prev_hash.clone(), DIFICULTY as usize)?;

        self.db.push(block)
    }

    pub fn balance_of(&mut self, wallet: &Wallet) -> io::Result<Val> {
        let mut balance = 0;
        
        while let Some(block) = self.next() {
            for transaction in block.get_transactions() {
                if transaction.is_receiver(wallet) {
                    balance += transaction.get_value().clone();
                } else if transaction.is_sender(wallet) {
                    let value = transaction.get_value().clone();
                    if value > balance {
                        self.db.reset();
                        return Err(io::Error::new(io::ErrorKind::InvalidData, format!("can\'t suptract {} from {}", value, balance)));
                    }

                    balance -= value;
                }
            }
        }

        Ok(balance)
    }

    pub fn is_valid(&mut self, verbose: bool) -> bool {
        while let Some(block) = self.next() {
            if !block.has_valid_transactions(verbose) {
                self.db.reset();
                return false;
            }

            let expected_hash = block.create_hash();
            let hash = block.get_hash();
            if hash != &expected_hash {
                if verbose {
                    verbose!(format!("block ({}) hash {} is invalid, expected {}",
                        ut_to_str(block.get_timestamp().clone()), hex(hash), hex(&expected_hash)))
                }

                self.db.reset();
                return false;
            }
        }
        
        // Check for prev_hash validance
        let mut prev_block = match self.next() {
            Some(prev_block) => prev_block,
            None => {
                if verbose {
                    verbose!("since blockchain dont have genesis block it is invalid");
                }
                self.db.reset();
                return false;
            },
        }; // skip first block
        
        while let Some(current) = self.next() {
            let expected_prev_hash = prev_block.get_hash();
            let prev_hash = current.get_prev_hash();
            if prev_hash != expected_prev_hash {
                if verbose {
                    verbose!(format!("block ({}) previous hash {} is invaid, expected {}",
                        ut_to_str(current.get_timestamp().clone()), hex(prev_hash), hex(expected_prev_hash)))
                }

                self.db.reset();
                return false;
            }

            prev_block = current   
        }

        true
    }

    pub fn len(&self) -> usize {
        self.db.len().clone()
    }
}

impl Iterator for Blockchain {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        self.db.next()
    }
}