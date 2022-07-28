use crate::error::Error;
use kelk::alloc::string::String;
use kelk::context::Context;
use kelk::kelk_entry;
use kelk::storage::str::StorageString;

#[kelk_entry]
pub fn instantiate(ctx: Context, _: ()) -> Result<(), Error> {
    let mut storage_string = StorageString::create(ctx.storage, 64)?;
    ctx.storage.fill_stack_at(1, storage_string.offset())?;
    storage_string.set_string("hello world!")?;

    Ok(())
}

#[kelk_entry]
pub fn process(ctx: Context, msg: String) -> Result<(), Error> {
    let storage_string_offset = ctx.storage.read_stack_at(1)?;
    let mut storage_string = StorageString::load(ctx.storage, storage_string_offset)?;
    storage_string.set_string(&msg)?;

    Ok(())
}

#[kelk_entry]
pub fn query(ctx: Context, _: ()) -> Result<String, Error> {
    let storage_string_offset = ctx.storage.read_stack_at(1)?;
    let storage_string = StorageString::load(ctx.storage, storage_string_offset)?;
    let msg = storage_string.get_string()?;

    Ok(msg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kelk::alloc::string::ToString;
    use kelk::mock::mock_context;

    #[test]
    fn test_instantiate() {
        let ctx = mock_context(1024);
        instantiate(ctx.as_ref(), ()).unwrap();
        let msg = query(ctx.as_ref(), ()).unwrap();
        assert_eq!(msg, "hello world!".to_string());
    }

    #[test]
    fn test_process() {
        let ctx = mock_context(1024);
        instantiate(ctx.as_ref(), ()).unwrap();
        process(ctx.as_ref(), "foo".to_string()).unwrap();
        let msg = query(ctx.as_ref(), ()).unwrap();
        assert_eq!(msg, "foo".to_string());
    }
}
