use crate::error::Error;
use kelk::blockchain::address::{Address, ADDRESS_ZERO};
use kelk::context::Context;
use kelk::storage::bst::StorageBST;
use kelk::storage::codec::Codec;
use kelk::storage::str::StorageString;
use kelk::Codec;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Codec)]
struct PairAddress(Address, Address);

pub(crate) struct ERC721<'a> {
    // context to access to storage and blockchain APIs
    ctx: Context<'a>,

    // Mapping from token ID to owner address
    owners: StorageBST<'a, i64, Address>,

    // Mapping owner address to token count
    balances: StorageBST<'a, Address, i64>,

    // Mapping from token ID to approved address
    token_approvals: StorageBST<'a, i64, Address>,

    // Mapping from owner to operator approvals
    operator_approvals: StorageBST<'a, PairAddress, bool>,

    name: StorageString<'a>,
    symbol: StorageString<'a>,
}

impl<'a> ERC721<'a> {
    pub fn instantiate(
        ctx: Context<'a>,
        token_name: &str,
        token_symbol: &str,
    ) -> Result<Self, Error> {
        let balances = StorageBST::create(ctx.storage)?;
        let token_approvals = StorageBST::create(ctx.storage)?;
        let operator_approvals = StorageBST::create(ctx.storage)?;
        let owners = StorageBST::create(ctx.storage)?;
        let mut name = StorageString::create(ctx.storage, token_name.len() as u32)?;
        let mut symbol = StorageString::create(ctx.storage, token_symbol.len() as u32)?;

        name.set_string(token_name)?;
        symbol.set_string(token_symbol)?;

        ctx.storage.fill_stack_at(1, balances.offset())?;
        ctx.storage.fill_stack_at(2, name.offset())?;
        ctx.storage.fill_stack_at(3, symbol.offset())?;
        ctx.storage.fill_stack_at(4, token_approvals.offset())?;
        ctx.storage.fill_stack_at(5, operator_approvals.offset())?;
        ctx.storage.fill_stack_at(6, owners.offset())?;

        Ok(Self {
            ctx,
            balances,
            name,
            symbol,
            token_approvals,
            operator_approvals,
            owners,
        })
    }

    pub fn load(ctx: Context<'a>) -> Result<Self, Error> {
        let balances_offset = ctx.storage.read_stack_at(1)?;
        let name_offset = ctx.storage.read_stack_at(2)?;
        let symbol_offset = ctx.storage.read_stack_at(3)?;
        let token_approvals_offset = ctx.storage.read_stack_at(4)?;
        let operator_approvals_offset = ctx.storage.read_stack_at(5)?;
        let owners_offset = ctx.storage.read_stack_at(6)?;

        let balances = StorageBST::load(ctx.storage, balances_offset)?;
        let name = StorageString::load(ctx.storage, name_offset)?;
        let symbol = StorageString::load(ctx.storage, symbol_offset)?;

        let token_approvals = StorageBST::load(ctx.storage, token_approvals_offset)?;
        let operator_approvals = StorageBST::load(ctx.storage, operator_approvals_offset)?;
        let owners = StorageBST::load(ctx.storage, owners_offset)?;

        Ok(Self {
            ctx,
            balances,
            name,
            symbol,
            token_approvals,
            operator_approvals,
            owners,
        })
    }

    pub fn name(&self) -> Result<String, Error> {
        Ok(self.name.get_string()?)
    }

    pub fn symbol(&self) -> Result<String, Error> {
        Ok(self.symbol.get_string()?)
    }

    pub fn balance_of(&self, addr: &Address) -> Result<i64, Error> {
        let balance = self.balances.find(addr)?.unwrap_or(0);
        Ok(balance)
    }

    pub fn owner_of(&self, token_id: &i64) -> Result<Address, Error> {
        let owner = self.owners.find(token_id)?;
        Ok(owner.unwrap())
    }

    pub fn transfer(&mut self, to: &Address, token_id: &i64) -> Result<(), Error> {
        let from: Address = self.ctx.blockchain.get_message_sender()?;
        self.transfer_from(&from, to, token_id)
    }

    pub fn approve(&mut self, to: &Address, token_id: &i64) -> Result<(), Error> {
        let owner: Address = self.ctx.blockchain.get_message_sender()?;

        let balance = self.balances.find(&owner)?.unwrap_or(0);
        if balance.ne(&0) {
            self._approved(&to, token_id)?;
        }
        Ok(())
    }

    fn _approved(&mut self, to: &Address, token_id: &i64) -> Result<(), Error> {
        if to.ne(&ADDRESS_ZERO) {
            self.token_approvals.insert(token_id.clone(), to.clone())?;
            Ok(())
        } else {
            Err(Error::InvalidMsg)
        }
    }

    pub fn get_approved(&self, token_id: &i64) -> Result<Address, Error> {
        let approved = self.token_approvals.find(token_id).unwrap();
        Ok(approved.unwrap())
    }

    pub fn set_approval_for_all(
        &mut self,
        operator: &Address,
        approved: &bool,
    ) -> Result<(), Error> {
        let owner = self.ctx.blockchain.get_message_sender()?;
        let balance = self.balances.find(&owner).unwrap().unwrap_or(0);
        if balance.ne(&0) {
            self._setted_approval_for_all(&owner, operator, approved)?;
        }

        Ok(())
    }

    fn _setted_approval_for_all(
        &mut self,
        owner: &Address,
        operator: &Address,
        approved: &bool,
    ) -> Result<(), Error> {
        self.operator_approvals.insert(
            PairAddress(owner.clone(), operator.clone()),
            approved.clone(),
        )?;
        Ok(())
    }

    pub fn is_approved_for_all(&self, owner: &Address, operator: &Address) -> Result<bool, Error> {
        let approved = self
            .operator_approvals
            .find(&PairAddress(owner.clone(), operator.clone()))
            .unwrap()
            .unwrap_or(false);
        Ok(approved)
    }

    pub fn transfer_from(
        &mut self,
        from: &Address,
        to: &Address,
        token_id: &i64,
    ) -> Result<(), Error> {
        let owner = self.owners.find(token_id).unwrap();

        if owner.unwrap() != from.clone() {
            return Err(Error::InsufficientAmount);
        }

        let tx_balance = self.balances.find(from).unwrap().unwrap_or(0);
        let rx_balance = self.balances.find(to).unwrap().unwrap_or(0);

        self.balances.insert(from.clone(), tx_balance - 1)?;
        self.balances.insert(to.clone(), rx_balance + 1)?;

        // self.owners.offset(token_id, from.clone())?;
        self.owners.insert(token_id.clone(), to.clone())?;

        Ok(())
    }

    pub fn mint(&mut self, addr: &Address, token_id: &i64) -> Result<(), Error> {
        if addr.ne(&ADDRESS_ZERO) {
            self.owners.insert(token_id.clone(), addr.clone())?;

            let tx_balance = self.balances.find(addr).unwrap().unwrap_or(0);
            self.balances.insert(addr.clone(), tx_balance + 1)?;
        }
        Ok(())
    }

    pub fn burn(&mut self, addr: &Address, token_id: &i64) -> Result<(), Error> {
        // if addr.ne(&ADDRESS_ZERO) {
        //     let acc_balance = self.balance_of(addr)?;
        //     if acc_balance.lt(amount) {
        //         return Err(Error::InsufficientAmount);
        //     }
        //     self.total_supply -= amount;
        // }
        Ok(())
    }
}
