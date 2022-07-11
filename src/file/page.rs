use anyhow::Result;
use chrono::{Datelike, NaiveDate};
use itertools::izip;
use std::fmt;
use std::mem;
use thiserror::Error;

#[derive(Debug, Default, Hash, PartialEq, Eq, Ord, PartialOrd, Clone)]
pub struct Page {
    pub bb: Vec<u8>,
}

impl fmt::Display for Page {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ bb: {:?} }}", self.bb)
    }
}

#[derive(Debug, Error)]
pub enum PageError {
    #[error("buffer size exceeded")]
    BufferSizeExceeded,
}

impl From<Vec<u8>> for Page {
    fn from(bb: Vec<u8>) -> Self {
        Self { bb }
    }
}

impl<const N: usize> From<[u8; N]> for Page {
    fn from(bb: [u8; N]) -> Self {
        Self { bb: bb.to_vec() }
    }
}

impl<const N: usize> From<&[u8; N]> for Page {
    fn from(bb: &[u8; N]) -> Self {
        Self { bb: bb.to_vec() }
    }
}

impl FromIterator<u8> for Page {
    fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
        Self {
            bb: iter.into_iter().collect(),
        }
    }
}

macro_rules! get_type {
    ($type_name:ty, $str_name:ident) => {
        pub fn $str_name(&self, offset: usize) -> Result<$type_name> {
            let $str_name = mem::size_of::<$type_name>();

            if offset + $str_name - 1 < self.bb.len() {
                let bytes = &self.bb[offset..offset + $str_name];
                Ok(<$type_name>::from_be_bytes((*bytes).try_into()?))
            } else {
                Err(From::from(PageError::BufferSizeExceeded))
            }
        }
    };
}

macro_rules! set_type {
    ($type_name:ty, $str_name:ident) => {
        pub fn $str_name(&mut self, offset: usize, n: $type_name) -> Result<usize> {
            let bytes = n.to_be_bytes();

            if offset + bytes.len() - 1 < self.bb.len() {
                for (b, added) in izip!(&mut self.bb[offset..offset + bytes.len()], &bytes) {
                    *b = *added;
                }

                Ok(offset + bytes.len())
            } else {
                Err(From::from(PageError::BufferSizeExceeded))
            }
        }
    };
}

impl Page {
    pub fn with_capacity(blocksize: usize) -> Self {
        Self {
            bb: vec![0; blocksize],
        }
    }

    get_type!(u8, get_u8);
    get_type!(u16, get_u16);
    get_type!(u32, get_u32);
    get_type!(u64, get_u64);

    set_type!(i8, get_i8);
    get_type!(i16, get_i16);
    get_type!(i32, get_i32);
    get_type!(i64, get_i64);

    set_type!(u8, set_u8);
    set_type!(u16, set_u16);
    set_type!(u32, set_u32);
    get_type!(u64, set_u64);

    set_type!(i8, set_i8);
    set_type!(i16, set_i16);
    set_type!(i32, set_i32);
    get_type!(i64, set_i64);

    pub fn get_bytes(&self, offset: usize) -> Result<&[u8]> {
        let len = self.get_i32(offset)? as usize;
        let new_offset = offset + mem::size_of::<i32>();

        if new_offset + len - 1 < self.bb.len() {
            Ok(&self.bb[new_offset..new_offset + len])
        } else {
            Err(From::from(PageError::BufferSizeExceeded))
        }
    }

    pub fn set_bytes(&mut self, offset: usize, b: &[u8]) -> Result<usize> {
        if offset + mem::size_of::<i32>() + b.len() - 1 < self.bb.len() {
            let new_offset = self.set_i32(offset, b.len() as i32)?;
            for (p, added) in izip!(&mut self.bb[new_offset..new_offset + b.len()], b) {
                *p = *added
            }

            Ok(new_offset + b.len())
        } else {
            Err(From::from(PageError::BufferSizeExceeded))
        }
    }

    pub fn get_string(&self, offset: usize) -> Result<String> {
        let bytes = self.get_bytes(offset)?;
        Ok(String::from_utf8(bytes.to_vec())?)
    }

    pub fn set_string(&mut self, offset: usize, s: String) -> Result<usize> {
        self.set_bytes(offset, s.as_bytes())
    }

    pub fn max_length(size: usize) -> usize {
        mem::size_of::<i32>() + (size * mem::size_of::<u8>())
    }

    pub fn contents(&self) -> Vec<u8> {
        self.bb.clone()
    }

    pub fn get_bool(&self, offset: usize) -> Result<bool> {
        self.get_u8(offset).map(|n| n != 0)
    }

    pub fn set_bool(&mut self, offset: usize, b: bool) -> Result<usize> {
        self.set_u8(offset, if b { 1 } else { 0 })
    }

    pub fn get_date(&self, offset: usize) -> Result<NaiveDate> {
        self.get_u32(offset).map(|ymd| {
            let y = ymd >> 16;
            let m = (ymd >> 8) & 255;
            let d = ymd & 255;
            NaiveDate::from_ymd(y as i32, m, d)
        })
    }

    pub fn set_date(&mut self, offset: usize, d: NaiveDate) -> Result<usize> {
        let ymd = (((d.year() as u32) << 8) + d.month()) << (8 + d.day());
        self.set_u32(offset, ymd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from() {
        assert_eq!(Page::from(vec![1]), Page { bb: vec![1] });
        assert_eq!(Page::from(&[1u8]), Page { bb: vec![1] });
        assert_eq!(Page::from([1u8]), Page { bb: vec![1] });
        assert_eq!(Page::from_iter([1u8].into_iter()), Page { bb: vec![1] });
    }
}
