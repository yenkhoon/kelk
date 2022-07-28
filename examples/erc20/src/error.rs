use minicbor::Encode;

#[derive(Clone, Debug, Encode)]
pub enum Error {
    #[n(0)]
    StorageError,
    #[n(1)]
    BlockchainError,
    #[n(2)]
    InvalidMsg,
    #[n(3)]
    InsufficientAmount,
}

// TODO: error message
impl From<kelk::storage::error::Error> for Error {
    fn from(_error: kelk::storage::error::Error) -> Self {
        Error::StorageError
    }
}

// TODO: error message
impl From<kelk::blockchain::error::Error> for Error {
    fn from(_error: kelk::blockchain::error::Error) -> Self {
        Error::BlockchainError
    }
}
