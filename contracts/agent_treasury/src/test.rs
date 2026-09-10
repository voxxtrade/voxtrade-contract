#![allow(unexpected_cfgs)]
#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, testutils::Ledger, Address, BytesN, Env};

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
        sequence_number: 17281,
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

#[test]
fn test_already_initialized() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let agent = Address::generate(&env);
    let treasury_id = env.register_contract(None, AgentTreasury);
    let treasury = AgentTreasuryClient::new(&env, &treasury_id);

    // First init -> Success
    let res = treasury.try_init(&admin, &agent, &100_i128);
    assert!(res.is_ok());

    // Second init -> Must fail with AlreadyInitialized
    let rogue_admin = Address::generate(&env);
    let rogue_agent = Address::generate(&env);
    let res2 = treasury.try_init(&rogue_admin, &rogue_agent, &9999_i128);
    assert_eq!(
        res2.unwrap_err().unwrap(),
        TreasuryError::AlreadyInitialized
    );
}

#[test]
fn test_unauthorized_agent() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let authorized_agent = Address::generate(&env);
    let rogue_agent = Address::generate(&env);
    let token = Address::generate(&env);
    let seller = Address::generate(&env);

    let treasury_id = env.register_contract(None, AgentTreasury);
    let treasury = AgentTreasuryClient::new(&env, &treasury_id);
    let escrow_id = env.register_contract(None, MockEscrow);

    treasury.init(&admin, &authorized_agent, &100_i128);

    let hash_lock = BytesN::from_array(&env, &[0; 32]);
    let timeout = 1000;

    // Call from rogue agent -> Must fail with Unauthorized
    let res = treasury.try_execute_x402_lock(
        &rogue_agent,
        &token,
        &escrow_id,
        &seller,
        &10_i128,
        &hash_lock,
        &timeout,
    );
    assert_eq!(res.unwrap_err().unwrap(), TreasuryError::Unauthorized);
}

#[test]
fn test_exact_limit_boundary() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let agent = Address::generate(&env);
    let token = Address::generate(&env);
    let seller = Address::generate(&env);

    let treasury_id = env.register_contract(None, AgentTreasury);
    let treasury = AgentTreasuryClient::new(&env, &treasury_id);
    let escrow_id = env.register_contract(None, MockEscrow);

    treasury.init(&admin, &agent, &100_i128);

    let hash_lock = BytesN::from_array(&env, &[0; 32]);
    let timeout = 1000;

    // Spend exactly 100 -> Must succeed
    let res = treasury.try_execute_x402_lock(
        &agent, &token, &escrow_id, &seller, &100_i128, &hash_lock, &timeout,
    );
    assert!(res.is_ok());

    // Spend 1 more -> Must fail with LimitExceeded
    let res2 = treasury.try_execute_x402_lock(
        &agent, &token, &escrow_id, &seller, &1_i128, &hash_lock, &timeout,
    );
    assert_eq!(res2.unwrap_err().unwrap(), TreasuryError::LimitExceeded);
}

#[test]
fn test_multiple_spends_same_day() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let agent = Address::generate(&env);
    let token = Address::generate(&env);
    let seller = Address::generate(&env);

    let treasury_id = env.register_contract(None, AgentTreasury);
    let treasury = AgentTreasuryClient::new(&env, &treasury_id);
    let escrow_id = env.register_contract(None, MockEscrow);

    treasury.init(&admin, &agent, &50_i128);

    let hash_lock = BytesN::from_array(&env, &[0; 32]);
    let timeout = 1000;

    // Spend 10 five times (total 50)
    for _ in 0..5 {
        treasury.execute_x402_lock(
            &agent, &token, &escrow_id, &seller, &10_i128, &hash_lock, &timeout,
        );
    }

    // 6th spend -> LimitExceeded
    let res = treasury.try_execute_x402_lock(
        &agent, &token, &escrow_id, &seller, &1_i128, &hash_lock, &timeout,
    );
    assert_eq!(res.unwrap_err().unwrap(), TreasuryError::LimitExceeded);
}

#[test]
fn test_multi_day_rolling_windows() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let agent = Address::generate(&env);
    let token = Address::generate(&env);
    let seller = Address::generate(&env);

    let treasury_id = env.register_contract(None, AgentTreasury);
    let treasury = AgentTreasuryClient::new(&env, &treasury_id);
    let escrow_id = env.register_contract(None, MockEscrow);

    treasury.init(&admin, &agent, &100_i128);

    let hash_lock = BytesN::from_array(&env, &[0; 32]);
    let timeout = 1000;

    // Day 0: Spend 80
    treasury.execute_x402_lock(
        &agent, &token, &escrow_id, &seller, &80_i128, &hash_lock, &timeout,
    );

    // Day 1 (seq 17280): Spend 90 -> Success
    env.ledger().set(soroban_sdk::testutils::LedgerInfo {
        timestamp: 86400,
        protocol_version: 20,
        sequence_number: 17280,
        network_id: [0; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 1,
        min_persistent_entry_ttl: 1,
        max_entry_ttl: 1000,
    });
    treasury.execute_x402_lock(
        &agent, &token, &escrow_id, &seller, &90_i128, &hash_lock, &timeout,
    );

    // Day 1: Try spend 20 -> Fail (90 + 20 = 110 > 100)
    let res = treasury.try_execute_x402_lock(
        &agent, &token, &escrow_id, &seller, &20_i128, &hash_lock, &timeout,
    );
    assert_eq!(res.unwrap_err().unwrap(), TreasuryError::LimitExceeded);

    // Day 3 (seq 17280 * 3 = 51840): Skip a day, spend 100 -> Success
    env.ledger().set(soroban_sdk::testutils::LedgerInfo {
        timestamp: 86400 * 3,
        protocol_version: 20,
        sequence_number: 51840,
        network_id: [0; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 1,
        min_persistent_entry_ttl: 1,
        max_entry_ttl: 1000,
    });
    let res2 = treasury.try_execute_x402_lock(
        &agent, &token, &escrow_id, &seller, &100_i128, &hash_lock, &timeout,
    );
    assert!(res2.is_ok());
}

#[contract]
pub struct MockToken;
#[contractimpl]
impl MockToken {
    pub fn transfer(_env: Env, _from: Address, _to: Address, _amount: i128) {}
}

#[test]
fn test_update_limit() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let agent = Address::generate(&env);
    let rogue = Address::generate(&env);

    let treasury_id = env.register_contract(None, AgentTreasury);
    let treasury = AgentTreasuryClient::new(&env, &treasury_id);

    treasury.init(&admin, &agent, &100_i128);

    // Admin updates limit to 200 -> Success
    treasury.update_limit(&admin, &200_i128);
    let config = treasury.get_config();
    assert_eq!(config.daily_limit, 200_i128);

    // Rogue fails to update limit
    let res = treasury.try_update_limit(&rogue, &500_i128);
    assert_eq!(res.unwrap_err().unwrap(), TreasuryError::Unauthorized);

    // Invalid limit <= 0
    let res2 = treasury.try_update_limit(&admin, &0_i128);
    assert_eq!(res2.unwrap_err().unwrap(), TreasuryError::InvalidAmount);
}

#[test]
fn test_update_agent_key() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let agent1 = Address::generate(&env);
    let agent2 = Address::generate(&env);
    let rogue = Address::generate(&env);

    let treasury_id = env.register_contract(None, AgentTreasury);
    let treasury = AgentTreasuryClient::new(&env, &treasury_id);

    treasury.init(&admin, &agent1, &100_i128);

    // Admin updates agent key to agent2 -> Success
    treasury.update_agent_key(&admin, &agent2);
    let config = treasury.get_config();
    assert_eq!(config.agent_key, agent2);

    // Rogue fails to update agent key
    let res = treasury.try_update_agent_key(&rogue, &agent1);
    assert_eq!(res.unwrap_err().unwrap(), TreasuryError::Unauthorized);
}

#[test]
fn test_withdraw() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let agent = Address::generate(&env);
    let recipient = Address::generate(&env);
    let rogue = Address::generate(&env);
    let token = env.register_contract(None, MockToken);

    let treasury_id = env.register_contract(None, AgentTreasury);
    let treasury = AgentTreasuryClient::new(&env, &treasury_id);

    treasury.init(&admin, &agent, &100_i128);

    // Admin withdraws -> Success
    let res = treasury.try_withdraw(&admin, &token, &recipient, &50_i128);
    assert!(res.is_ok());

    // Rogue withdraws -> Unauthorized
    let res2 = treasury.try_withdraw(&rogue, &token, &recipient, &50_i128);
    assert_eq!(res2.unwrap_err().unwrap(), TreasuryError::Unauthorized);

    // Invalid amount <= 0
    let res3 = treasury.try_withdraw(&admin, &token, &recipient, &-10_i128);
    assert_eq!(res3.unwrap_err().unwrap(), TreasuryError::InvalidAmount);
}

#[test]
fn test_getters() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let agent = Address::generate(&env);

    let treasury_id = env.register_contract(None, AgentTreasury);
    let treasury = AgentTreasuryClient::new(&env, &treasury_id);

    treasury.init(&admin, &agent, &100_i128);

    let config = treasury.get_config();
    assert_eq!(config.admin, admin);
    assert_eq!(config.agent_key, agent);
    assert_eq!(config.daily_limit, 100_i128);

    let daily_spend = treasury.get_daily_spend();
    assert_eq!(daily_spend.amount_spent, 0);
    assert_eq!(daily_spend.day, 0);
}
