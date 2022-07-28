use crate::error::Error;
use kelk::blockchain::address::{Address, ADDRESS_ZERO};
use kelk::context::Context;
use kelk::storage::bst::StorageBST;
use kelk::storage::codec::Codec;
use kelk::storage::str::StorageString;
use kelk::Codec;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Codec)]
struct PairAddress(Address, Address);

pub(crate) struct ERC20<'a> {
    // context to access to storage and blockchain APIs
    ctx: Context<'a>,

    /// Mapping from owner to number of owned token.
    balances: StorageBST<'a, Address, i64>,

    /// Mapping of the token amount which an account is allowed to withdraw
    /// from another account.
    allowances: StorageBST<'a, PairAddress, i64>,

    total_supply: i64,
    name: StorageString<'a>,
    symbol: StorageString<'a>,
}

impl<'a> ERC20<'a> {
    pub fn instantiate(
        ctx: Context<'a>,
        token_name: &str,
        token_symbol: &str,
        total_supply: i64,
    ) -> Result<Self, Error> {
        let mut balances = StorageBST::create(ctx.storage)?;
        let allowances = StorageBST::create(ctx.storage)?;
        let mut name = StorageString::create(ctx.storage, token_name.len() as u32)?;
        let mut symbol = StorageString::create(ctx.storage, token_symbol.len() as u32)?;

        let owner = ctx.blockchain.get_message_sender()?;
        balances.insert(owner, total_supply).unwrap();

        let total_supply_offset = ctx.storage.allocate(i64::PACKED_LEN)?;
        ctx.storage.write_i64(total_supply_offset, &total_supply)?;
        name.set_string(token_name)?;
        symbol.set_string(token_symbol)?;

        ctx.storage.fill_stack_at(1, balances.offset())?;
        ctx.storage.fill_stack_at(2, allowances.offset())?;
        ctx.storage.fill_stack_at(3, name.offset())?;
        ctx.storage.fill_stack_at(4, symbol.offset())?;
        ctx.storage.fill_stack_at(5, total_supply_offset)?;

        Ok(Self {
            ctx,
            balances,
            allowances,
            total_supply,
            name,
            symbol,
        })
    }

    pub fn load(ctx: Context<'a>) -> Result<Self, Error> {
        let balances_offset = ctx.storage.read_stack_at(1)?;
        let allowances_offset = ctx.storage.read_stack_at(2)?;
        let name_offset = ctx.storage.read_stack_at(3)?;
        let symbol_offset = ctx.storage.read_stack_at(4)?;
        let total_supply_offset = ctx.storage.read_stack_at(5)?;

        let balances = StorageBST::load(ctx.storage, balances_offset)?;
        let allowances = StorageBST::load(ctx.storage, allowances_offset)?;
        let name = StorageString::load(ctx.storage, name_offset)?;
        let symbol = StorageString::load(ctx.storage, symbol_offset)?;

        let total_supply = ctx.storage.read_i64(total_supply_offset)?;

        Ok(Self {
            ctx,
            balances,
            allowances,
            total_supply,
            name,
            symbol,
        })
    }

    pub fn name(&self) -> Result<String, Error> {
        Ok(self.name.get_string()?)
    }

    pub fn symbol(&self) -> Result<String, Error> {
        Ok(self.symbol.get_string()?)
    }

    pub fn total_supply(&self) -> Result<i64, Error> {
        Ok(self.total_supply)
    }

    pub fn balance_of(&self, addr: &Address) -> Result<i64, Error> {
        let balance = self.balances.find(addr).unwrap().unwrap_or(0);
        Ok(balance)
    }

    pub fn transfer(&mut self, to: &Address, amount: &i64) -> Result<(), Error> {
        let from: Address = self.ctx.blockchain.get_message_sender()?;
        self.transfer_from(&from, to, amount)
    }

    pub fn allowance(&self, owner: &Address, spender: &Address) -> i64 {
        self.allowances
            .find(&PairAddress(owner.clone(), spender.clone()))
            .unwrap()
            .unwrap_or(0)
    }

    pub fn approve(&mut self, spender: &Address, amount: &i64) -> Result<(), Error> {
        let owner: Address = self.ctx.blockchain.get_message_sender()?;
        self._approved(&owner, spender, amount)?;
        Ok(())
    }

    fn _approved(&mut self, owner: &Address, spender: &Address, amount: &i64) -> Result<(), Error> {
        if owner.ne(&ADDRESS_ZERO) && spender.ne(&ADDRESS_ZERO) {
            self.allowances
                .insert(PairAddress(owner.clone(), spender.clone()), *amount)?;
            Ok(())
        } else {
            Ok(())
        }
    }

    pub fn transfer_from(
        &mut self,
        from: &Address,
        to: &Address,
        amount: &i64,
    ) -> Result<(), Error> {
        let tx_balance = self.balances.find(from).unwrap().unwrap_or(0);
        let rx_balance = self.balances.find(to).unwrap().unwrap_or(0);

        if tx_balance.lt(amount) {
            return Err(Error::InsufficientAmount);
        }
        self.balances.insert(from.clone(), tx_balance - amount)?;
        self.balances.insert(to.clone(), rx_balance + amount)?;

        Ok(())
    }

    pub fn mint(&mut self, addr: &Address, amount: &i64) -> Result<(), Error> {
        if addr.ne(&ADDRESS_ZERO) {
            self.total_supply += amount;
        }
        Ok(())
    }

    pub fn burn(&mut self, addr: &Address, amount: &i64) -> Result<(), Error> {
        if addr.ne(&ADDRESS_ZERO) {
            let acc_balance = self.balance_of(addr)?;
            if acc_balance.lt(amount) {
                return Err(Error::InsufficientAmount);
            }
            self.total_supply -= amount;
        }
        Ok(())
    }

    pub fn increase_allowance(&mut self, spender: &Address, amount: &i64) -> Result<(), Error> {
        let from: Address = self.ctx.blockchain.get_message_sender()?;
        let allowance = self.allowance(&from, spender);
        self._approved(&from, spender, &(allowance + amount))?;
        Ok(())
    }

    pub fn decrease_allowance(&mut self, spender: &Address, amount: &i64) -> Result<(), Error> {
        let owner: Address = self.ctx.blockchain.get_message_sender()?;
        let current_allowance: i64 = self.allowance(&owner, spender);
        if current_allowance.lt(amount) {
            return Err(Error::InsufficientAmount);
        }
        let allowance = self.allowance(&owner, spender);
        self._approved(&owner, spender, &(allowance - amount))?;
        Ok(())
    }

    pub fn spend_allowance(
        &mut self,
        owner: &Address,
        spender: &Address,
        amount: &i64,
    ) -> Result<(), Error> {
        let current_allowance: i64 = self.allowance(owner, spender);
        if current_allowance.lt(amount) {
            return Err(Error::InsufficientAmount);
        }

        self._approved(owner, spender, &(current_allowance - amount))?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "./erc20_test.rs"]
mod tests;
