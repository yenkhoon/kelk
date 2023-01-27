use crate::erc721::ERC721;
use crate::error::Error;
use crate::message::{InstantiateMsg, ProcMsg, QueryMsg, QueryRsp};
use kelk::context::Context;
use kelk::kelk_entry;

/*
instantiate creates a new contract and deployment code.
*/
#[kelk_entry]
pub fn instantiate(ctx: Context, msg: InstantiateMsg) -> Result<(), Error> {
    ERC721::instantiate(ctx, &msg.name, &msg.symbol)?;
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
    let mut token = ERC721::load(ctx)?;
    match &msg {
        ProcMsg::Transfer { to, amount } => token.transfer(to, amount),
        ProcMsg::TransferFrom { from, to, amount } => token.transfer_from(from, to, amount),
        ProcMsg::Approve { to, amount } => token.approve(to, amount),
        ProcMsg::Mint { addr, amount } => token.mint(addr, amount),
        ProcMsg::Burn { addr, amount } => token.burn(addr, amount),
    }
}

/*
query executes the contract associated with the addr with the given input
as parameters while disallowing any modifications to the state during the call.
*/
#[kelk_entry]
pub fn query(ctx: Context, msg: QueryMsg) -> Result<QueryRsp, Error> {
    let token = ERC721::load(ctx)?;
    let res = match &msg {
        QueryMsg::Name => QueryRsp::Name { res: token.name()? },
        QueryMsg::Symbol => QueryRsp::Symbol {
            res: token.symbol()?,
        },
        QueryMsg::Balance { addr } => QueryRsp::Balance {
            res: token.balance_of(addr)?,
        },
        QueryMsg::Approve { to, token_id } => QueryRsp::Allowance {
            res: token.approve(to, token_id),
        },
    };

    Ok(res)
}
