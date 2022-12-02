//! Imported WASM functions
//!
//! Contract actors can call this imported function to interact with the
//! blockchain and the storage file.

use crate::api::{BlockchainAPI, StorageAPI};
use crate::error::HostError;
use crate::memory::Pointer;
use alloc::vec::Vec;
use core::any::Any;
use minicbor::{Decode, Encode};

#[cfg(not(test))]
#[link(wasm_import_module = "pactus")]
extern "C" {
    /// writes data at given offset of storage file.
    ///
    /// # Arguments
    ///
    /// `offset` is the offset of data in the storage file.
    /// `ptr` is the location in sandbox memory where data should be read from.
    /// `len` is the length of data.
    ///
    /// If the operation is successful it returns 0, otherwise it reruns the error code.
    fn write_storage(offset: u32, ptr: u32, len: u32) -> i32;
    /// reads data from the given offset of storage file.
    ///
    /// # Arguments
    ///
    /// `offset` is the offset of data in the storage file.
    /// `ptr` is the location in sandbox memory where data should be written to.
    /// `len` is the length of data.
    ///
    /// If the operation is successful it returns 0, otherwise it reruns the error code.
    fn read_storage(offset: u32, ptr: u32, len: u32) -> i32;

    /// gets parameter value from the host.
    ///
    /// # Arguments
    ///
    /// `param_id` is the parameter ID that is known for the host.
    /// `ptr` is the location in sandbox memory where data should be written to.
    /// `len` is the length of data.
    ///
    /// If the operation is successful it returns 0, otherwise it reruns the error code.
    fn get_param(param_id: u32, ptr: *mut u32, len: *mut u32) -> i32;
}

/// TODO: rename it to API or ExternalAPI or Externality

/// The instant of Kelk.
pub struct Kelk {}

impl Kelk {
    /// creates a new instance of Kelk.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Kelk {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageAPI for Kelk {
    fn read(&self, offset: u32, data: &mut [u8]) -> Result<(), HostError> {
        let ptr = data.as_ptr() as u32;
        let len = data.len() as u32;

        let code = unsafe { read_storage(offset, ptr, len) };
        if code != 0 {
            return Err(HostError { code });
        }
        Ok(())
    }

    fn write(&self, offset: u32, data: &[u8]) -> Result<(), HostError> {
        let ptr = data.as_ptr() as u32;
        let len = data.len() as u32;

        let code = unsafe { write_storage(offset, ptr, len) };
        if code != 0 {
            return Err(HostError { code });
        }
        Ok(())
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl BlockchainAPI for Kelk {
    fn get_param(&self, param_id: u32) -> Result<Vec<u8>, HostError> {
        let mut len = 0;
        let mut ptr = 0;

        let code = unsafe { get_param(param_id, &mut ptr, &mut len) };
        if code != 0 {
            return Err(HostError { code });
        }
        let slice = unsafe { core::slice::from_raw_parts(ptr as *const u8, len as usize) };
        Ok(slice.to_vec())
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

/// `do_instantiate` should be wrapped in an external "C" export,
/// containing a contract-specific function as arg.
pub fn do_instantiate<'a, T, D: Decode<'a, ()>, E: Encode<()>>(
    instantiate_fn: &dyn Fn(T, D) -> Result<(), E>,
    ctx: T,
    msg_ptr: u64,
) -> u64 {
    do_execute(instantiate_fn, ctx, msg_ptr)
}

/// `do_process` should be wrapped in an external "C" export,
/// containing a contract-specific function as arg.
pub fn do_process<'a, T, D: Decode<'a, ()>, E: Encode<()>>(
    process_fn: &dyn Fn(T, D) -> Result<(), E>,
    ctx: T,
    msg_ptr: u64,
) -> u64 {
    do_execute(process_fn, ctx, msg_ptr)
}

/// `do_query` should be wrapped in an external "C" export,
/// containing a contract-specific function as arg.
pub fn do_query<'a, T, D: Decode<'a, ()>, R: Encode<()>, E: Encode<()>>(
    query_fn: &dyn Fn(T, D) -> Result<R, E>,
    ctx: T,
    msg_ptr: u64,
) -> u64 {
    do_execute(query_fn, ctx, msg_ptr)
}

fn do_execute<'a, T, D: Decode<'a, ()>, R: Encode<()>, E: Encode<()>>(
    func: &dyn Fn(T, D) -> Result<R, E>,
    ctx: T,
    msg_ptr: u64,
) -> u64 {
    let ptr = Pointer::from_u64(msg_ptr);
    let buf = unsafe { ptr.to_slice() };
    let msg = minicbor::decode(buf).expect("Decoding failed");
    let res = func(ctx, msg);
    let mut vec = crate::alloc::vec::Vec::new();
    minicbor::encode(res, &mut vec).expect("Encoding failed");

    Pointer::release_buffer(vec).as_u64()
}

/// For testing
#[cfg(test)]
pub unsafe fn write_storage(_offset: u32, _ptr: u32, _len: u32) -> i32 {
    0
}

/// For testing
#[cfg(test)]
pub unsafe fn read_storage(_offset: u32, _ptr: u32, _len: u32) -> i32 {
    0
}

/// For testing
#[cfg(test)]
pub unsafe fn get_param(_param_id: u32, _ptr: *mut u32, _len: *mut u32) -> i32 {
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alloc::vec;
    use wasm_bindgen_test::*;

    // Uncomment this test if should_panic supported by wasm_bindgen_test.
    // https://github.com/rustwasm/wasm-bindgen/issues/2286
    //
    // #[wasm_bindgen_test]
    // #[should_panic]
    // fn test_allocation() {
    //     let ptr = allocate(1);
    //     deallocate(ptr);

    //     // Should panic here, because the pointer is freed before
    //     deallocate(ptr);
    // }

    #[wasm_bindgen_test]
    fn test_instantiate() {
        let msg_data = vec![0x00]; // http://cbor.me/?bytes=8100
        let msg_ptr = Pointer::release_buffer(msg_data);

        let res_ptr = do_instantiate(
            &|_: (), _: i32| -> Result<(), i32> { Ok(()) },
            (),
            msg_ptr.as_u64(),
        );

        let res_data = unsafe { Pointer::from_u64(res_ptr).to_slice() };
        assert_eq!(res_data, vec![0x82, 0x00, 0x80]); // Result::Ok(()) -> http://cbor.me/?bytes=820080
    }

    #[wasm_bindgen_test]
    fn test_instantiate_error() {
        let msg_data = vec![0x00]; // http://cbor.me/?bytes=8100
        let msg_ptr = Pointer::release_buffer(msg_data);

        let res_ptr = do_instantiate(
            &|_: (), _: i32| -> Result<(), i32> { Err(0x0e) },
            (),
            msg_ptr.as_u64(),
        );

        let res_data = unsafe { Pointer::from_u64(res_ptr).to_slice() };
        assert_eq!(res_data, vec![0x82, 0x01, 0x0e]); // Result::Err(0x0e) -> http://cbor.me/?bytes=82010e
    }

    #[wasm_bindgen_test]
    fn test_process() {
        let msg_data = vec![0x00]; // http://cbor.me/?bytes=8100
        let msg_ptr = Pointer::release_buffer(msg_data);

        let res_ptr = do_process(
            &|_: (), _: i32| -> Result<(), i32> { Ok(()) },
            (),
            msg_ptr.as_u64(),
        );

        let res_data = unsafe { Pointer::from_u64(res_ptr).to_slice() };
        assert_eq!(res_data, vec![0x82, 0x00, 0x80]); // Result::Ok(()) -> http://cbor.me/?bytes=820080
    }

    #[wasm_bindgen_test]
    fn test_process_error() {
        let msg_data = vec![0x00]; // http://cbor.me/?bytes=8100
        let msg_ptr = Pointer::release_buffer(msg_data);

        let res_ptr = do_process(
            &|_: (), _: i32| -> Result<(), i32> { Err(0x0e) },
            (),
            msg_ptr.as_u64(),
        );

        let res_data = unsafe { Pointer::from_u64(res_ptr).to_slice() };
        assert_eq!(res_data, vec![0x82, 0x01, 0x0e]); // Result::Err(0x0e) -> http://cbor.me/?bytes=82010e
    }

    #[wasm_bindgen_test]
    fn test_query() {
        let msg_data = vec![0x00]; // http://cbor.me/?bytes=8100
        let msg_ptr = Pointer::release_buffer(msg_data);

        let res_ptr = do_query(
            &|_: (), _: i32| -> Result<&str, i32> { Ok("foo") },
            (),
            msg_ptr.as_u64(),
        );

        let res_data = unsafe { Pointer::from_u64(res_ptr).to_slice() };
        assert_eq!(res_data, vec![0x82, 0x00, 0x63, 0x66, 0x6f, 0x6f]); // Result::Ok("foo") -> http://cbor.me/?bytes=0x820063666f6f
    }

    #[wasm_bindgen_test]
    fn test_query_error() {
        let msg_data = vec![0x00]; // http://cbor.me/?bytes=8100
        let msg_ptr = Pointer::release_buffer(msg_data);

        let res_ptr = do_query(
            &|_: (), _: i32| -> Result<&str, i32> { Err(0x0e) },
            (),
            msg_ptr.as_u64(),
        );

        let res_data = unsafe { Pointer::from_u64(res_ptr).to_slice() };
        assert_eq!(res_data, vec![0x82, 0x01, 0x0e]); // Result::Err(0x0e) -> http://cbor.me/?bytes=82010e
    }
}
