use kelk::blockchain::address::Address;
use minicbor::{Decode, Encode};

#[derive(Clone, Debug, Encode, Decode)]
pub enum ProcMsg {
    #[n(0)]
    TransferFrom {
        #[n(0)]
        from: Address,
        #[n(1)]
        to: Address,
        #[n(2)]
        amount: i64,
    },
    #[n(1)]
    Transfer {
        #[n(0)]
        to: Address,
        #[n(1)]
        amount: i64,
    },
    #[n(2)]
    Approved {
        #[n(0)]
        to: Address,
        #[n(1)]
        token_id: i64,
    },
    #[n(3)]
    ApprovedForAll {
        #[n(0)]
        operator: Address,
        #[n(1)]
        approved: bool,
    },
    #[n(4)]
    Mint {
        #[n(0)]
        addr: Address,
        #[n(1)]
        amount: i64,
    },
    #[n(5)]
    Burn {
        #[n(0)]
        addr: Address,
        #[n(1)]
        amount: i64,
    },
}
#[derive(Clone, Debug, Encode, Decode)]
pub struct InstantiateMsg {
    #[n(0)]
    pub name: String,
    #[n(1)]
    pub symbol: String,
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum QueryMsg {
    #[n(0)]
    Name,
    #[n(1)]
    Symbol,
    #[n(2)]
    Balance {
        #[n(0)]
        addr: Address,
    },
    #[n(3)]
    Approved {
        #[n(0)]
        token_id: i64,
    },
    #[n(4)]
    ApprovedForAll {
        #[n(0)]
        operator: Address,
        #[n(1)]
        owner: Address,
    },
    #[n(5)]
    OwnerOf {
        #[n(0)]
        token_id: i64,
    },
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum QueryRsp {
    #[n(0)]
    Name {
        #[n(0)]
        res: String,
    },
    #[n(1)]
    Symbol {
        #[n(1)]
        res: String,
    },
    #[n(2)]
    Balance {
        #[n(2)]
        res: i64,
    },
    #[n(3)]
    Approved {
        #[n(2)]
        res: Address,
    },
    #[n(4)]
    ApprovedForAll {
        #[n(0)]
        res: bool,
    },
    #[n(5)]
    OwnerOf {
        #[n(0)]
        res: Address,
    },
}
