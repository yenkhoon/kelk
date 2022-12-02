//! Kelk-allocator providing memory allocator support for smart contracts in [Pactus](https://pactus.org/) blockchain.
//!
#![cfg_attr(no_std, feature(core_intrinsics, lang_items, alloc_error_handler))]
// #![cfg_attr(no_std, feature(core_intrinsics, lang_items, alloc_error_handler))]

// Use `wee_alloc` as the global allocator.
// #[cfg(no_std)]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Need to provide a tiny `panic` implementation for `#![no_std]`.
/// This translates into an `unreachable` instruction that will
/// raise a `trap` the WebAssembly execution if we panic at runtime.
#[cfg(no_std)]
#[panic_handler]
#[no_mangle]
pub fn panic(_info: &::core::panic::PanicInfo) -> ! {
    ::core::intrinsics::abort();
}

/// Need to provide an allocation error handler which just aborts
/// the execution with trap.
#[cfg(no_std)]
#[alloc_error_handler]
#[no_mangle]
fn oom(_: core::alloc::Layout) -> ! {
    ::core::intrinsics::abort();
}

/// Needed for non-wasm targets.
#[cfg(no_std)]
#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn eh_personality() {}
