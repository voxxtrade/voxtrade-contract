#![no_std]

pub mod errors;
pub mod types;

#[cfg(test)]
mod test;

use crate::errors::EscrowError;
use crate::types::{DataKey, Escrow};
use soroban_sdk::{contract, contractimpl, crypto::Hash, token, Address, BytesN, Env, Symbol};

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
        timeout_ledger: u32,
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
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_EXTEND, TTL_EXTEND);

        env.events().publish(
            (Symbol::new(&env, "FundsLocked"),),
            (next_nonce, buyer, seller, amount),
        );
        next_nonce
    }

    pub fn claim(env: Env, escrow_id: u64, preimage: BytesN<32>) -> Result<(), EscrowError> {
        let key = DataKey::Escrow(escrow_id);
        let mut escrow: Escrow = env.storage().persistent().get(&key).unwrap();

        if escrow.resolved {
            return Err(EscrowError::AlreadyResolved);
        }

        if env.ledger().sequence() > escrow.timeout_ledger {
            return Err(EscrowError::TimeoutReached);
        }

        let computed_hash = env.crypto().sha256(&preimage);
        if computed_hash != escrow.hash_lock {
            return Err(EscrowError::InvalidHash);
        }

        escrow.resolved = true;
        env.storage().persistent().set(&key, &escrow);
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_EXTEND, TTL_EXTEND);

        token::Client::new(&env, &escrow.token).transfer(
            &env.current_contract_address(),
            &escrow.seller,
            &escrow.amount,
        );
        env.events().publish(
            (Symbol::new(&env, "FundsClaimed"),),
            (escrow_id, escrow.seller),
        );
        Ok(())
    }

    pub fn refund(env: Env, escrow_id: u64) -> Result<(), EscrowError> {
        let key = DataKey::Escrow(escrow_id);
        let mut escrow: Escrow = env.storage().persistent().get(&key).unwrap();

        if escrow.resolved {
            return Err(EscrowError::AlreadyResolved);
        }

        if env.ledger().sequence() <= escrow.timeout_ledger {
            return Err(EscrowError::TimeoutNotReached);
        }

        escrow.resolved = true;
        env.storage().persistent().set(&key, &escrow);
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_EXTEND, TTL_EXTEND);

        token::Client::new(&env, &escrow.token).transfer(
            &env.current_contract_address(),
            &escrow.buyer,
            &escrow.amount,
        );
        Ok(())
    }
}
