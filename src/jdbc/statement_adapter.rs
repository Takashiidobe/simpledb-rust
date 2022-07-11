use anyhow::Result;
use thiserror::Error;

// use super::result_set_adapter::ResultSetAdapter;

#[derive(Debug, Error, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum StatementError {
    #[error("runtime error")]
    RuntimeError,
}

// pub trait StatementAdapter<'a> {
//     type Set: ResultSetAdapter;
//     type Affected;
//     type Res;
//
//     fn execute_query(&'a mut self) -> Result<Self::Set>;
//     fn execute_update(&mut self) -> Result<Self::Affected>;
//     fn close(&mut self) -> Result<Self::Res>;
// }
