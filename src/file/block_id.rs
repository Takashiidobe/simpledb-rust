use std::fmt;

#[derive(Debug, Default, Hash, PartialEq, Eq, Ord, PartialOrd, Clone)]
pub struct BlockId {
    filename: String,
    blknum: i32,
}

impl BlockId {
    pub fn new<S: Into<String>>(filename: S, blknum: i32) -> Self {
        BlockId {
            filename: filename.into(),
            blknum,
        }
    }

    pub fn file_name(&self) -> String {
        self.filename.clone()
    }

    pub fn number(&self) -> i32 {
        self.blknum
    }
}

impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[file {}, block {}]", self.filename, self.blknum)
    }
}
