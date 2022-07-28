//! Modules definition for blockchain libraries

pub mod address;
pub mod error;
pub mod mock;

use self::address::Address;
use self::error::Error;
use alloc::boxed::Box;
use alloc::vec::Vec;
use kelk_env::BlockchainAPI;

const PARAM_ID_LAST_BLOCK_HASH: u32 = 0x0001;
const PARAM_ID_LAST_BLOCK_TIME: u32 = 0x0002;
const PARAM_ID_TRANSACTION_SIGNER: u32 = 0x0010;

/// Blockchain object
pub struct Blockchain {
    /// Blockchain APIs that are provided by the host
    api: Box<dyn BlockchainAPI>,
}

impl Blockchain {
    /// creates a new instance of Blockchain
    pub fn new(api: Box<dyn BlockchainAPI>) -> Self {
        Self { api }
    }

    pub(crate) fn api_mut(&mut self) -> &mut Box<dyn BlockchainAPI> {
        &mut self.api
    }

    /// returns the last block hash
    pub fn get_last_block_hash(&self) -> Result<Vec<u8>, Error> {
        Ok(self.api.get_param(PARAM_ID_LAST_BLOCK_HASH)?)
    }

    /// returns the last block time
    pub fn get_last_block_time(&self) -> Result<Vec<u8>, Error> {
        Ok(self.api.get_param(PARAM_ID_LAST_BLOCK_TIME)?)
    }

    /// returns the message sender address
    pub fn get_message_sender(&self) -> Result<Address, Error> {
        let data = self.api.get_param(PARAM_ID_TRANSACTION_SIGNER)?;
        Address::from_bytes(&data)
    }
}
