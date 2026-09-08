#![no_std]
use soroban_sdk::{contract, contractimpl, token, Address, BytesN, Env, Symbol};

mod errors;
#[cfg(test)]
mod test;
mod types;

use errors::EscrowError;
use types::Escrow;

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
    ) -> Result<u64, EscrowError> {
        buyer.require_auth();

        if amount <= 0 {
            return Err(EscrowError::InvalidAmount);
        }
        if timeout_ledger <= env.ledger().sequence() {
            return Err(EscrowError::InvalidTimeout);
        }

        token::Client::new(&env, &token).transfer(&buyer, &env.current_contract_address(), &amount);

        let nonce_key = Symbol::new(&env, "nonce");
        let mut nonce: u64 = env.storage().instance().get(&nonce_key).unwrap_or(0);
        nonce += 1;
        env.storage().instance().set(&nonce_key, &nonce);

        let escrow = Escrow {
            buyer,
            seller,
            token,
            amount,
            hash_lock,
            timeout_ledger,
            resolved: false,
        };

        env.storage()
            .persistent()
            .set(&types::DataKey::Escrow(nonce), &escrow);
        env.storage()
            .persistent()
            .extend_ttl(&types::DataKey::Escrow(nonce), 100_000, 100_000);

        Ok(nonce)
    }

    pub fn claim(env: Env, escrow_id: u64, preimage: BytesN<32>) -> Result<(), EscrowError> {
        let key = types::DataKey::Escrow(escrow_id);
        let mut escrow: Escrow = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(EscrowError::NotFound)?;

        if escrow.resolved {
            return Err(EscrowError::AlreadyResolved);
        }

        let computed_bytesn: soroban_sdk::BytesN<32> = env.crypto().sha256(&preimage);
        

        if computed_bytesn != escrow.hash_lock {
            return Err(EscrowError::HashMismatch);
        }

        escrow.resolved = true;
        env.storage().persistent().set(&key, &escrow);

        token::Client::new(&env, &escrow.token).transfer(
            &env.current_contract_address(),
            &escrow.seller,
            &escrow.amount,
        );
        Ok(())
    }

    pub fn refund(env: Env, escrow_id: u64) -> Result<(), EscrowError> {
        let key = types::DataKey::Escrow(escrow_id);
        let mut escrow: Escrow = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(EscrowError::NotFound)?;

        if escrow.resolved {
            return Err(EscrowError::AlreadyResolved);
        }
        if env.ledger().sequence() < escrow.timeout_ledger {
            return Err(EscrowError::TimeoutNotReached);
        }

        escrow.resolved = true;
        env.storage().persistent().set(&key, &escrow);

        token::Client::new(&env, &escrow.token).transfer(
            &env.current_contract_address(),
            &escrow.buyer,
            &escrow.amount,
        );
        Ok(())
    }
}

