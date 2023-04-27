mod transaction;
mod blockchain;
mod wallet;
mod encode;
mod block;
mod debug;
mod db;

use wallet::*;
use blockchain::Blockchain;
use transaction::Transaction;

const DATABASE: &'static str = "bc.bin";

fn main() {
    let mut valid: bool = true;
    while valid {
        println!("-----------------------");
        let owner = Wallet::new().expect("can\'t create wallet");
        let receiver = Wallet::new().expect("can\'t create wallet");
    
        let mut blockchain = Blockchain::load(DATABASE).expect("can\'t  load blockchain");
    
        let transaction = Transaction::new(&owner, &receiver, 10);
        blockchain.mine(&owner, &[transaction]).expect("can\'t mine a block");

        valid = blockchain.is_valid(true)
    }

    println!("blockchain is invalid")
}