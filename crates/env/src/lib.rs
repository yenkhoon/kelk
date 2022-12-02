//! Kelk-env is a Low-level interface for interacting with Tanour (Wasm executor) in Zarb blockchain.
//!
#![no_std]
#![deny(
    missing_docs,
    bad_style,
    bare_trait_objects,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates
)]

pub mod api;
pub mod error;

pub use api::BlockchainAPI;
pub use api::StorageAPI;
pub use error::HostError;

#[cfg(target_arch = "wasm32")]
mod memory;

#[cfg(target_arch = "wasm32")]
pub mod import;

#[cfg(target_arch = "wasm32")]
pub mod export;

#[cfg(target_arch = "wasm32")]
pub use crate::import::{do_instantiate, do_process, do_query};

pub extern crate alloc;
