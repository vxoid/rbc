use crate::transaction::Transaction;
use crate::block::*;
use crate::warning;
use std::io::{Write, Read, Seek, self};
use std::mem;
use std::fs;

pub struct BlockDB {
    file: fs::File,
    len: usize
}

impl BlockDB {
    pub fn open(path: &str) -> io::Result<Self> {
        let file: fs::File = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        let mut len: usize = 0;
        let mut db: Self = Self { file, len };

        while let Some(_) = db.next() {
            len += 1
        }

        db.len = len;

        Ok(db)
    }

    pub fn push(&mut self, block: Block) -> io::Result<()> {
        let mut writer: io::BufWriter<&mut fs::File> = io::BufWriter::new(&mut self.file);

        writer.seek(io::SeekFrom::End(0))?;

        let bytes: Vec<u8> = block.into();
        writer.write_all(&bytes)?;

        writer.flush()?;
        writer.seek(io::SeekFrom::Start(0))?;

        self.len += 1;

        Ok(())
    }

    pub fn last_block(&mut self) -> Option<Block> {
        let mut last: Option<Block> = None;

        while let Some(block) = self.next() {
            last = Some(block);    
        }

        last
    }

    pub fn reset(&mut self) {
        self.file.seek(io::SeekFrom::Start(0)).expect("can\'t seek the file");
    }

    pub fn len(&self) -> &usize {
        &self.len
    }
}

impl Iterator for BlockDB {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        let mut batch: [u8; BLOCK_STATIC_SIZE+mem::size_of::<usize>()] = [0; BLOCK_STATIC_SIZE+mem::size_of::<usize>()];

        let bytes_read: usize = match self.file.read(&mut batch) {
            Ok(bytes_read) => bytes_read,
            Err(err) => {
                warning!(format!("can\'t read file due to {}", err));
                return None;
            },
        };

        if bytes_read == 0 {
            self.reset();
            return None;
        }

        let block_static: &BlockStatic = unsafe {
            &*(batch.as_ptr() as *const BlockStatic)
        };
        let transactions_len: usize = unsafe { 
            *((batch.as_ptr() as usize + BLOCK_STATIC_SIZE) as *const usize)
        };

        let transactions: Vec<Transaction> = Transaction::from_file(&mut self.file, transactions_len);

        Some(Block::from((block_static.clone(), transactions)))
    }
}