//! Storage error types

use alloc::string::String;
use core::fmt::Debug;

/// A general list of Storage error
#[derive(Debug)]
pub enum Error {
    /// Host error code
    HostError(i32),

    /// stack overflow error
    StackOverflow,

    /// Invalid offset
    InvalidOffset(u32),

    /// Capacity is full
    OutOfCapacity,

    /// Generic error
    GenericError(String),
}

impl From<kelk_env::error::HostError> for Error {
    fn from(error: kelk_env::error::HostError) -> Self {
        Error::HostError(error.code)
    }
}
