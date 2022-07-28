use super::*;
use kelk::mock::{mock_context, MockContext};

fn setup(ctx: &mut MockContext) -> (ERC20, Address) {
    let owner = ctx.mocked_blockchain().generate_new_address();
    ctx.mocked_blockchain().set_msg_sender(&owner);
    ERC20::instantiate(ctx.as_ref(), "test-erc20", "$$$", 2000).unwrap();
    let erc20 = ERC20::load(ctx.as_ref()).unwrap();
    (erc20, owner)
}

#[test]
fn test_instantiate() {
    let mut ctx = mock_context(1024 * 1024);

    let (erc20, owner) = setup(&mut ctx);
    assert_eq!(erc20.name().unwrap(), "test-erc20".to_string());
    assert_eq!(erc20.symbol().unwrap(), "$$$".to_string());
    assert_eq!(erc20.balance_of(&owner).unwrap(), 2000);
    assert_eq!(erc20.total_supply().unwrap(), 2000);
}

#[test]
fn test_transfer() {
    let mut ctx = mock_context(1024 * 1024);

    let addr_1 = ctx.mocked_blockchain().generate_new_address();
    let addr_2 = ctx.mocked_blockchain().generate_new_address();
    let (mut erc20, owner) = setup(&mut ctx);
    assert!(erc20.transfer(&addr_1, &10).is_ok());
    assert!(erc20.transfer_from(&addr_1, &addr_2, &20).is_err());
    assert!(erc20.transfer_from(&addr_1, &addr_2, &5).is_ok());
    assert_eq!(erc20.balance_of(&addr_1).unwrap(), 5);
    assert_eq!(erc20.balance_of(&addr_2).unwrap(), 5);
    assert_eq!(erc20.balance_of(&owner).unwrap(), 1990);
}

#[test]
fn test_allowance() {
    let mut ctx = mock_context(1024 * 1024);
    let addr_1 = ctx.mocked_blockchain().generate_new_address();

    let (mut erc20, owner) = setup(&mut ctx);
    assert!(erc20.increase_allowance(&addr_1, &10).is_ok());
    assert_eq!(erc20.allowance(&owner, &addr_1), 10);
    assert!(erc20.decrease_allowance(&addr_1, &20).is_err());
    assert!(erc20.spend_allowance(&owner, &addr_1, &5).is_ok());
    assert_eq!(erc20.allowance(&owner, &addr_1), 5);
}
