#![allow(unexpected_cfgs)]
#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, testutils::Ledger, Address, BytesN, Env, contract, contractimpl};

#[contract]
pub struct MockToken;
#[contractimpl]
impl MockToken {
    pub fn transfer(_env: Env, _from: Address, _to: Address, _amount: i128) {}
}

#[test]
fn test_lock_and_claim() {
    let env = Env::default();
    env.mock_all_auths();

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let token = env.register_contract(None, MockToken);

    let escrow_id = env.register_contract(None, X402Escrow);
    let escrow = X402EscrowClient::new(&env, &escrow_id);

    let preimage = BytesN::from_array(&env, &[1; 32]);
    let preimage_bytes: soroban_sdk::Bytes = preimage.clone().into();
    let hash_lock: BytesN<32> = env.crypto().sha256(&preimage_bytes);
    let timeout = 1000;

    let id = escrow.lock_funds(&buyer, &seller, &token, &100_i128, &hash_lock, &timeout);

    // Claim
    escrow.claim(&id, &preimage);
}

#[test]
fn test_timeout_refund() {
    let env = Env::default();
    env.mock_all_auths();

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let token = env.register_contract(None, MockToken);

    let escrow_id = env.register_contract(None, X402Escrow);
    let escrow = X402EscrowClient::new(&env, &escrow_id);

    let preimage = BytesN::from_array(&env, &[1; 32]);
    let preimage_bytes: soroban_sdk::Bytes = preimage.clone().into();
    let hash_lock: BytesN<32> = env.crypto().sha256(&preimage_bytes);
    let timeout = 100;

    let id = escrow.lock_funds(&buyer, &seller, &token, &100_i128, &hash_lock, &timeout);

    // Attempt claim after timeout
    env.ledger().set(soroban_sdk::testutils::LedgerInfo {
        timestamp: 0,
        protocol_version: 20,
        sequence_number: 101, // Past timeout
        network_id: [0; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 1,
        min_persistent_entry_ttl: 1,
        max_entry_ttl: 1000,
    });

    let res = escrow.try_claim(&id, &preimage);
    assert_eq!(res.unwrap_err().unwrap(), EscrowError::TimeoutReached);

    escrow.refund(&id);
}
