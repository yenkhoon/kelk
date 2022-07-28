use crate::erc20::ERC20;
use crate::error::Error;
use crate::message::{InstantiateMsg, ProcMsg, QueryMsg, QueryRsp};
use kelk::context::Context;
use kelk::kelk_entry;

/*
instantiate creates a new contract and deployment code.
*/
#[kelk_entry]
pub fn instantiate(ctx: Context, msg: InstantiateMsg) -> Result<(), Error> {
    ERC20::instantiate(ctx, &msg.name, &msg.symbol, msg.total_supply)?;
    Ok(())
}

/*
process executes the contract associated with the addr with the given input as
parameters. It also handles any necessary value transfer required and takes
the necessary steps to create accounts and reverses the state in case of an
execution error or failed value transfer.
*/
#[kelk_entry]
pub fn process(ctx: Context, msg: ProcMsg) -> Result<(), Error> {
    let mut token = ERC20::load(ctx)?;
    match &msg {
        ProcMsg::Transfer { to, amount } => token.transfer(to, amount),
        ProcMsg::TransferFrom { from, to, amount } => token.transfer_from(from, to, amount),
        ProcMsg::Approve { to, amount } => token.approve(to, amount),
        ProcMsg::Mint { addr, amount } => token.mint(addr, amount),
        ProcMsg::Burn { addr, amount } => token.burn(addr, amount),
        ProcMsg::IncreaseAllowance { spender, amount } => token.increase_allowance(spender, amount),
        ProcMsg::DecreaseAllowance { spender, amount } => token.decrease_allowance(spender, amount),
        ProcMsg::SpendAllowance {
            owner,
            spender,
            amount,
        } => token.spend_allowance(owner, spender, amount),
    }
}

/*
query executes the contract associated with the addr with the given input
as parameters while disallowing any modifications to the state during the call.
*/
#[kelk_entry]
pub fn query(ctx: Context, msg: QueryMsg) -> Result<QueryRsp, Error> {
    let token = ERC20::load(ctx)?;
    let res = match &msg {
        QueryMsg::Name => QueryRsp::Name { res: token.name()? },
        QueryMsg::Symbol => QueryRsp::Symbol {
            res: token.symbol()?,
        },
        QueryMsg::TotalSupply => QueryRsp::TotalSupply {
            res: token.total_supply()?,
        },
        QueryMsg::Balance { addr } => QueryRsp::Balance {
            res: token.balance_of(addr)?,
        },
        QueryMsg::Allowance { owner, spender } => QueryRsp::Allowance {
            res: token.allowance(owner, spender),
        },
    };

    Ok(res)
}
