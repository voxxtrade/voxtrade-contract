#![no_std]
use soroban_sdk::{contract, contractimpl, token, Address, BytesN, Env};

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

        let mut nonce: u64 = env
            .storage()
            .persistent()
            .get(&types::DataKey::Nonce)
            .unwrap_or(0);
        nonce += 1;
        env.storage()
            .persistent()
            .set(&types::DataKey::Nonce, &nonce);

        let escrow = Escrow {
            buyer: buyer.clone(),
            seller: seller.clone(),
            token,
            amount,
            hash_lock: hash_lock.clone(),
            timeout_ledger,
            resolved: false,
        };

        env.storage()
            .persistent()
            .set(&types::DataKey::Escrow(nonce), &escrow);
        env.storage()
            .persistent()
            .extend_ttl(&types::DataKey::Escrow(nonce), 100_000, 100_000);

        env.events().publish(
            (soroban_sdk::Symbol::new(&env, "escrow"), soroban_sdk::Symbol::new(&env, "lock_funds")),
            (nonce, buyer, seller, amount, hash_lock, timeout_ledger),
        );

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

        if env.ledger().sequence() >= escrow.timeout_ledger {
            return Err(EscrowError::TimeoutReached);
        }

        let preimage_bytes: soroban_sdk::Bytes = preimage.clone().into();
        let computed_bytesn: soroban_sdk::BytesN<32> = env.crypto().sha256(&preimage_bytes);

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

        env.events().publish(
            (soroban_sdk::Symbol::new(&env, "escrow"), soroban_sdk::Symbol::new(&env, "claim")),
            (escrow_id, escrow.seller, escrow.amount, preimage),
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

        env.events().publish(
            (soroban_sdk::Symbol::new(&env, "escrow"), soroban_sdk::Symbol::new(&env, "refund")),
            (escrow_id, escrow.buyer, escrow.amount),
        );

        Ok(())
    }

    pub fn cancel_cooperative(env: Env, escrow_id: u64) -> Result<(), EscrowError> {
        let key = types::DataKey::Escrow(escrow_id);
        let mut escrow: Escrow = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(EscrowError::NotFound)?;

        if escrow.resolved {
            return Err(EscrowError::AlreadyResolved);
        }

        // Seller authorization is required to cooperatively forfeit and refund buyer
        escrow.seller.require_auth();

        escrow.resolved = true;
        env.storage().persistent().set(&key, &escrow);

        token::Client::new(&env, &escrow.token).transfer(
            &env.current_contract_address(),
            &escrow.buyer,
            &escrow.amount,
        );

        env.events().publish(
            (soroban_sdk::Symbol::new(&env, "escrow"), soroban_sdk::Symbol::new(&env, "cancel")),
            (escrow_id, escrow.buyer, escrow.seller, escrow.amount),
        );

        Ok(())
    }

    pub fn get_escrow(env: Env, escrow_id: u64) -> Result<Escrow, EscrowError> {
        let key = types::DataKey::Escrow(escrow_id);
        env.storage().persistent().get(&key).ok_or(EscrowError::NotFound)
    }
}
