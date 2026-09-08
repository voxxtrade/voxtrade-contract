#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

// We mock the escrow contract to test cross-contract calls
#[contract]
pub struct MockEscrow;
#[contractimpl]
impl MockEscrow {
    pub fn lock_funds(
        _env: Env,
        _buyer: Address,
        _seller: Address,
        _token: Address,
        _amount: i128,
        _hash_lock: BytesN<32>,
        _timeout: u32,
    ) -> u64 {
        1
    }
}

#[test]
fn test_rolling_limit() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let agent = Address::generate(&env);
    let token = Address::generate(&env);
    let seller = Address::generate(&env);

    let treasury_id = env.register_contract(None, AgentTreasury);
    let treasury = AgentTreasuryClient::new(&env, &treasury_id);

    let escrow_id = env.register_contract(None, MockEscrow);

    // Initial limit: 100 USDC
    treasury.init(&admin, &agent, &100_i128);

    let hash_lock = BytesN::from_array(&env, &[0; 32]);
    let timeout = 1000;

    // Spend 60 -> Success
    treasury.execute_x402_lock(
        &agent, &token, &escrow_id, &seller, &60_i128, &hash_lock, &timeout,
    );

    // Spend 50 -> Fail (60 + 50 = 110 > 100)
    let res = treasury.try_execute_x402_lock(
        &agent, &token, &escrow_id, &seller, &50_i128, &hash_lock, &timeout,
    );
    assert_eq!(res.unwrap_err().unwrap(), TreasuryError::LimitExceeded);

    // Advance ledger by 1 day and 1 second
    env.ledger().set(soroban_sdk::testutils::LedgerInfo {
        timestamp: 86401,
        protocol_version: 20,
        sequence_number: 100,
        network_id: [0; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 1,
        min_persistent_entry_ttl: 1,
        max_entry_ttl: 1000,
    });

    // Spend 50 again -> Success (Limit reset)
    treasury.execute_x402_lock(
        &agent, &token, &escrow_id, &seller, &50_i128, &hash_lock, &timeout,
    );
}
