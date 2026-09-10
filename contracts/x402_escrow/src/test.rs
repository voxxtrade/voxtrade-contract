#![allow(unexpected_cfgs)]
#![cfg(test)]

use super::*;
use soroban_sdk::{
    contract, contractimpl, testutils::Address as _, testutils::Ledger, Address, BytesN, Env,
};

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

#[test]
fn test_invalid_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let token = env.register_contract(None, MockToken);
    let escrow_id = env.register_contract(None, X402Escrow);
    let escrow = X402EscrowClient::new(&env, &escrow_id);

    let hash_lock = BytesN::from_array(&env, &[1; 32]);

    // Amount = 0
    let res = escrow.try_lock_funds(&buyer, &seller, &token, &0_i128, &hash_lock, &1000);
    assert_eq!(res.unwrap_err().unwrap(), EscrowError::InvalidAmount);

    // Negative amount
    let res2 = escrow.try_lock_funds(&buyer, &seller, &token, &-50_i128, &hash_lock, &1000);
    assert_eq!(res2.unwrap_err().unwrap(), EscrowError::InvalidAmount);
}

#[test]
fn test_invalid_timeout() {
    let env = Env::default();
    env.mock_all_auths();

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let token = env.register_contract(None, MockToken);
    let escrow_id = env.register_contract(None, X402Escrow);
    let escrow = X402EscrowClient::new(&env, &escrow_id);

    let hash_lock = BytesN::from_array(&env, &[1; 32]);

    // Set sequence to 50
    env.ledger().set(soroban_sdk::testutils::LedgerInfo {
        timestamp: 0,
        protocol_version: 20,
        sequence_number: 50,
        network_id: [0; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 1,
        min_persistent_entry_ttl: 1,
        max_entry_ttl: 1000,
    });

    // Timeout equal to current sequence
    let res = escrow.try_lock_funds(&buyer, &seller, &token, &100_i128, &hash_lock, &50);
    assert_eq!(res.unwrap_err().unwrap(), EscrowError::InvalidTimeout);

    // Timeout less than current sequence
    let res2 = escrow.try_lock_funds(&buyer, &seller, &token, &100_i128, &hash_lock, &49);
    assert_eq!(res2.unwrap_err().unwrap(), EscrowError::InvalidTimeout);
}

#[test]
fn test_escrow_not_found() {
    let env = Env::default();
    env.mock_all_auths();

    let escrow_id = env.register_contract(None, X402Escrow);
    let escrow = X402EscrowClient::new(&env, &escrow_id);

    let preimage = BytesN::from_array(&env, &[1; 32]);

    // Claim non-existent escrow
    let res = escrow.try_claim(&999_u64, &preimage);
    assert_eq!(res.unwrap_err().unwrap(), EscrowError::NotFound);

    // Refund non-existent escrow
    let res2 = escrow.try_refund(&999_u64);
    assert_eq!(res2.unwrap_err().unwrap(), EscrowError::NotFound);
}

#[test]
fn test_hash_mismatch() {
    let env = Env::default();
    env.mock_all_auths();

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let token = env.register_contract(None, MockToken);
    let escrow_id = env.register_contract(None, X402Escrow);
    let escrow = X402EscrowClient::new(&env, &escrow_id);

    let correct_preimage = BytesN::from_array(&env, &[1; 32]);
    let wrong_preimage = BytesN::from_array(&env, &[2; 32]);
    let preimage_bytes: soroban_sdk::Bytes = correct_preimage.clone().into();
    let hash_lock: BytesN<32> = env.crypto().sha256(&preimage_bytes);

    let id = escrow.lock_funds(&buyer, &seller, &token, &100_i128, &hash_lock, &1000);

    // Attempt claim with wrong preimage
    let res = escrow.try_claim(&id, &wrong_preimage);
    assert_eq!(res.unwrap_err().unwrap(), EscrowError::HashMismatch);

    // Then claim with correct preimage succeeds
    let res2 = escrow.try_claim(&id, &correct_preimage);
    assert!(res2.is_ok());
}

#[test]
fn test_double_claim_rejected() {
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

    let id = escrow.lock_funds(&buyer, &seller, &token, &100_i128, &hash_lock, &1000);

    // First claim -> Success
    escrow.claim(&id, &preimage);

    // Second claim -> AlreadyResolved
    let res = escrow.try_claim(&id, &preimage);
    assert_eq!(res.unwrap_err().unwrap(), EscrowError::AlreadyResolved);
}

#[test]
fn test_premature_refund_rejected() {
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

    let id = escrow.lock_funds(&buyer, &seller, &token, &100_i128, &hash_lock, &100);

    // Ledger sequence is 0, timeout is 100
    let res = escrow.try_refund(&id);
    assert_eq!(res.unwrap_err().unwrap(), EscrowError::TimeoutNotReached);
}

#[test]
fn test_double_refund_rejected() {
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

    let id = escrow.lock_funds(&buyer, &seller, &token, &100_i128, &hash_lock, &50);

    // Advance sequence past 50
    env.ledger().set(soroban_sdk::testutils::LedgerInfo {
        timestamp: 0,
        protocol_version: 20,
        sequence_number: 51,
        network_id: [0; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 1,
        min_persistent_entry_ttl: 1,
        max_entry_ttl: 1000,
    });

    // First refund -> Success
    escrow.refund(&id);

    // Second refund -> AlreadyResolved
    let res = escrow.try_refund(&id);
    assert_eq!(res.unwrap_err().unwrap(), EscrowError::AlreadyResolved);
}

#[test]
fn test_claim_after_refund_rejected() {
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

    let id = escrow.lock_funds(&buyer, &seller, &token, &100_i128, &hash_lock, &50);

    // Advance sequence past 50
    env.ledger().set(soroban_sdk::testutils::LedgerInfo {
        timestamp: 0,
        protocol_version: 20,
        sequence_number: 51,
        network_id: [0; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 1,
        min_persistent_entry_ttl: 1,
        max_entry_ttl: 1000,
    });

    escrow.refund(&id);

    // Try claim after refund -> AlreadyResolved
    let res = escrow.try_claim(&id, &preimage);
    assert_eq!(res.unwrap_err().unwrap(), EscrowError::AlreadyResolved);
}

#[test]
fn test_refund_after_claim_rejected() {
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

    let id = escrow.lock_funds(&buyer, &seller, &token, &100_i128, &hash_lock, &50);

    // Claim before timeout
    escrow.claim(&id, &preimage);

    // Advance sequence past 50
    env.ledger().set(soroban_sdk::testutils::LedgerInfo {
        timestamp: 0,
        protocol_version: 20,
        sequence_number: 51,
        network_id: [0; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 1,
        min_persistent_entry_ttl: 1,
        max_entry_ttl: 1000,
    });

    // Try refund after claim -> AlreadyResolved
    let res = escrow.try_refund(&id);
    assert_eq!(res.unwrap_err().unwrap(), EscrowError::AlreadyResolved);
}

#[test]
fn test_multiple_sequential_escrows() {
    let env = Env::default();
    env.mock_all_auths();

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let token = env.register_contract(None, MockToken);
    let escrow_id = env.register_contract(None, X402Escrow);
    let escrow = X402EscrowClient::new(&env, &escrow_id);

    let p1 = BytesN::from_array(&env, &[1; 32]);
    let h1: BytesN<32> = env.crypto().sha256(&p1.clone().into());

    let p2 = BytesN::from_array(&env, &[2; 32]);
    let h2: BytesN<32> = env.crypto().sha256(&p2.clone().into());

    let p3 = BytesN::from_array(&env, &[3; 32]);
    let h3: BytesN<32> = env.crypto().sha256(&p3.clone().into());

    let id1 = escrow.lock_funds(&buyer, &seller, &token, &10_i128, &h1, &1000);
    let id2 = escrow.lock_funds(&buyer, &seller, &token, &20_i128, &h2, &1000);
    let id3 = escrow.lock_funds(&buyer, &seller, &token, &30_i128, &h3, &1000);

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(id3, 3);

    // Resolve out of order: 2, then 1, then 3
    escrow.claim(&id2, &p2);
    escrow.claim(&id1, &p1);
    escrow.claim(&id3, &p3);
}

#[test]
fn test_cancel_cooperative() {
    let env = Env::default();
    env.mock_all_auths();

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let token = env.register_contract(None, MockToken);
    let escrow_id = env.register_contract(None, X402Escrow);
    let escrow = X402EscrowClient::new(&env, &escrow_id);

    let preimage = BytesN::from_array(&env, &[1; 32]);
    let hash_lock: BytesN<32> = env.crypto().sha256(&preimage.clone().into());

    let id = escrow.lock_funds(&buyer, &seller, &token, &100_i128, &hash_lock, &1000);

    // Cancel cooperatively before timeout
    escrow.cancel_cooperative(&id);

    // Check state is resolved
    let escrow_data = escrow.get_escrow(&id);
    assert!(escrow_data.resolved);

    // Trying to claim after cancel fails
    let res = escrow.try_claim(&id, &preimage);
    assert_eq!(res.unwrap_err().unwrap(), EscrowError::AlreadyResolved);

    // Trying to cancel again fails
    let res2 = escrow.try_cancel_cooperative(&id);
    assert_eq!(res2.unwrap_err().unwrap(), EscrowError::AlreadyResolved);
}

#[test]
fn test_get_escrow() {
    let env = Env::default();
    env.mock_all_auths();

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let token = env.register_contract(None, MockToken);
    let escrow_id = env.register_contract(None, X402Escrow);
    let escrow = X402EscrowClient::new(&env, &escrow_id);

    let preimage = BytesN::from_array(&env, &[1; 32]);
    let hash_lock: BytesN<32> = env.crypto().sha256(&preimage.clone().into());

    let id = escrow.lock_funds(&buyer, &seller, &token, &75_i128, &hash_lock, &500);

    let data = escrow.get_escrow(&id);
    assert_eq!(data.buyer, buyer);
    assert_eq!(data.seller, seller);
    assert_eq!(data.token, token);
    assert_eq!(data.amount, 75_i128);
    assert_eq!(data.hash_lock, hash_lock);
    assert_eq!(data.timeout_ledger, 500);
    assert!(!data.resolved);
}
