use super::block_id::BlockId;
use super::page::Page;
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::Path;
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Debug, Error)]
enum FileMgrError {
    #[error("parse failed")]
    ParseFailed,
    #[error("file access failed: {0}")]
    FileAccessFailed(String),
}

#[derive(Debug)]
pub struct FileMgr {
    pub db_directory: String,
    pub blocksize: i32,
    pub is_new: bool,
    pub open_files: HashMap<String, Arc<Mutex<File>>>,
}

impl FileMgr {
    pub fn new(db_directory: &str, blocksize: i32) -> Result<Self> {
        let path = Path::new(db_directory);
        let is_new = !path.exists();

        if is_new {
            fs::create_dir_all(path)?;
        }

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            let filename = match entry.file_name().into_string() {
                Ok(s) => s,
                Err(_) => return Err(From::from(FileMgrError::ParseFailed)),
            };

            if filename.starts_with("temp") {
                fs::remove_file(entry_path)?;
            }
        }

        Ok(Self {
            db_directory: String::from(db_directory),
            blocksize,
            is_new,
            open_files: HashMap::new(),
        })
    }

    pub fn read(&mut self, blk: &BlockId, p: &mut Page) -> anyhow::Result<()> {
        let offset = blk.number() * self.blocksize;

        if let Some(file) = self.get_file(blk.file_name().as_str()) {
            {
                let mut f = file.lock().unwrap();
                f.seek(SeekFrom::Start(offset.try_into().unwrap()))?;
                let read_len = f.read(&mut p.contents())?;
                let p_len = p.contents().len();

                if read_len < p_len {
                    let tmp = vec![0; p_len - read_len];
                    f.write_all(&tmp)?;

                    for i in read_len..p_len {
                        p.contents()[i] = 0;
                    }
                }
            }

            return Ok(());
        }

        Err(From::from(FileMgrError::FileAccessFailed(blk.file_name())))
    }

    pub fn write(&mut self, blk: &BlockId, p: &mut Page) -> anyhow::Result<()> {
        let offset = blk.number() * self.blocksize;

        if let Some(file) = self.get_file(blk.file_name().as_str()) {
            {
                let mut f = file.lock().unwrap();
                f.seek(SeekFrom::Start(offset.try_into().unwrap()))?;
                f.write_all(&p.contents())?;
                f.flush()?;
            }

            return Ok(());
        }

        Err(From::from(FileMgrError::FileAccessFailed(blk.file_name())))
    }

    pub fn append(&mut self, filename: &str) -> anyhow::Result<BlockId> {
        let new_blknum = self.length(filename)?;
        let blk = BlockId::new(filename, new_blknum);
        let b: Vec<u8> = vec![0u8; self.blocksize as usize];
        let offset = blk.number() * self.blocksize;

        if let Some(file) = self.get_file(blk.file_name().as_str()) {
            {
                let mut f = file.lock().unwrap();
                f.seek(SeekFrom::Start(offset.try_into().unwrap()))?;
                f.write_all(&b)?;
                f.flush()?;
            }

            return Ok(blk);
        }

        Err(From::from(FileMgrError::FileAccessFailed(
            filename.to_string(),
        )))
    }

    pub fn length(&mut self, filename: &str) -> Result<i32> {
        let path = Path::new(&self.db_directory).join(&filename);
        self.configure_file_table(filename)?;
        let md = fs::metadata(&path)?;

        // ceil
        Ok((md.len() as i32 + self.blocksize - 1) / self.blocksize)
    }

    pub fn configure_file_table(&mut self, filename: &str) -> anyhow::Result<()> {
        let path = Path::new(&self.db_directory).join(&filename);

        self.open_files
            .entry(filename.to_string())
            .or_insert(Arc::new(Mutex::new(
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(&path)?,
            )));

        Ok(())
    }

    pub fn block_size(&self) -> i32 {
        self.blocksize
    }

    pub fn is_new(&self) -> bool {
        self.is_new
    }

    fn get_file(&mut self, filename: &str) -> Option<&mut Arc<Mutex<File>>> {
        let path = Path::new(&self.db_directory).join(filename);

        let f = self
            .open_files
            .entry(filename.to_string())
            .or_insert_with(|| {
                Arc::new(Mutex::new(
                    OpenOptions::new()
                        .read(true)
                        .write(true)
                        .create(true)
                        .open(&path)
                        .unwrap(),
                ))
            });

        Some(f)
    }
}
