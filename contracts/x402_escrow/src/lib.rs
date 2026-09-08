#![no_std]

pub mod errors;
pub mod types;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, Address, Env, BytesN, Symbol, token};
use crate::types::{DataKey, Escrow};
use crate::errors::EscrowError;

const TTL_EXTEND: u32 = 535680;

#[contract]
pub struct X402Escrow;

#[contractimpl]
impl X402Escrow {
    pub fn lock_funds(
        env: Env, 
        buyer: Address, 
        seller: Address, 
        token: Address, 
        amount: i128, 
        hash_lock: BytesN<32>, 
        timeout_ledger: u32
    ) -> u64 {
        buyer.require_auth();

        env.storage().instance().extend_ttl(TTL_EXTEND, TTL_EXTEND);

        if timeout_ledger <= env.ledger().sequence() {
            panic!("timeout already reached");
        }

        let nonce: u64 = env.storage().instance().get(&DataKey::Nonce).unwrap_or(0);
        let next_nonce = nonce + 1;
        env.storage().instance().set(&DataKey::Nonce, &next_nonce);

        token::Client::new(&env, &token).transfer(&buyer, &env.current_contract_address(), &amount);

        let escrow = Escrow {
            buyer: buyer.clone(),
            seller: seller.clone(),
            amount,
            token,
            hash_lock,
            timeout_ledger,
            resolved: false,
        };

        let key = DataKey::Escrow(next_nonce);
        env.storage().persistent().set(&key, &escrow);
        env.storage().persistent().extend_ttl(&key, TTL_EXTEND, TTL_EXTEND);

        env.events().publish((Symbol::new(&env, "FundsLocked"),), (next_nonce, buyer, seller, amount));
        next_nonce
    }
}

