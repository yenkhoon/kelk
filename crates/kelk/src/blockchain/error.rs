//! Blockchain error types

use alloc::string::String;
use core::fmt::Debug;

/// A general list of Blockchain error
#[derive(Debug)]
pub enum Error {
    /// Host error code
    HostError {
        /// Host error code
        code: i32,
    },

    /// Invalid length
    InvalidLength {
        /// The expected length(s).
        expected: usize,
        /// The invalid length found.
        found: usize,
    },

    /// Generic error
    GenericError(String),
}

impl From<kelk_env::error::HostError> for Error {
    fn from(error: kelk_env::error::HostError) -> Self {
        Error::HostError { code: error.code }
    }
}
