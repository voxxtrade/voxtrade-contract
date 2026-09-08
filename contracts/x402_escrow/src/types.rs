use soroban_sdk::{contracttype, Address, BytesN};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Escrow {
    pub buyer: Address,
    pub seller: Address,
    pub amount: i128,
    pub token: Address,
    pub hash_lock: BytesN<32>,
    pub timeout_ledger: u32,
    pub resolved: bool,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Escrow(u64), // Maps to Escrow struct
    Nonce,       // u64
}

