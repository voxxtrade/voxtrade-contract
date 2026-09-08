#![no_std]

pub mod errors;
pub mod types;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, Address, Env, BytesN, Symbol, vec};
use crate::types::DataKey;
use crate::errors::TreasuryError;

const TTL_EXTEND: u32 = 535680;
const SECONDS_IN_DAY: u64 = 86400;

#[contract]
pub struct AgentTreasury;

#[contractimpl]
impl AgentTreasury {
    pub fn init(env: Env, admin: Address, agent_key: Address, daily_limit: i128) -> Result<(), TreasuryError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(TreasuryError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::AgentKey, &agent_key);
        env.storage().instance().set(&DataKey::DailyLimit, &daily_limit);
        env.storage().instance().set(&DataKey::SpentToday, &0_i128);
        env.storage().instance().set(&DataKey::LastReset, &env.ledger().timestamp());
        
        env.storage().instance().extend_ttl(TTL_EXTEND, TTL_EXTEND);
        Ok(())
    }

    pub fn update_limit(env: Env, admin: Address, new_limit: i128) -> Result<(), TreasuryError> {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            return Err(TreasuryError::Unauthorized);
        }

        env.storage().instance().set(&DataKey::DailyLimit, &new_limit);
        env.storage().instance().extend_ttl(TTL_EXTEND, TTL_EXTEND);
        
        env.events().publish((Symbol::new(&env, "LimitUpdated"),), new_limit);
        Ok(())
    }

    pub fn execute_x402_lock(
        env: Env, 
        agent: Address, 
        token: Address, 
        escrow_contract: Address, 
        seller: Address, 
        amount: i128, 
        hash_lock: BytesN<32>, 
        timeout_ledger: u32
    ) -> Result<(), TreasuryError> {
        agent.require_auth();
        let stored_agent: Address = env.storage().instance().get(&DataKey::AgentKey).unwrap();
        if agent != stored_agent {
            return Err(TreasuryError::Unauthorized);
        }

        env.storage().instance().extend_ttl(TTL_EXTEND, TTL_EXTEND);

        if amount <= 0 {
            return Err(TreasuryError::InvalidAmount);
        }

        let mut spent_today: i128 = env.storage().instance().get(&DataKey::SpentToday).unwrap();
        let mut last_reset: u64 = env.storage().instance().get(&DataKey::LastReset).unwrap();
        let current_time = env.ledger().timestamp();

        if current_time > last_reset + SECONDS_IN_DAY {
            spent_today = 0;
            last_reset = current_time;
            env.storage().instance().set(&DataKey::LastReset, &last_reset);
        }

        let daily_limit: i128 = env.storage().instance().get(&DataKey::DailyLimit).unwrap();
        if spent_today + amount > daily_limit {
            return Err(TreasuryError::LimitExceeded);
        }

        spent_today += amount;
        env.storage().instance().set(&DataKey::SpentToday, &spent_today);

        let args = vec![
            &env,
            env.current_contract_address().into_val(&env),
            seller.into_val(&env),
            token.into_val(&env),
            amount.into_val(&env),
            hash_lock.into_val(&env),
            timeout_ledger.into_val(&env)
        ];
        
        let _escrow_id: u64 = env.invoke_contract(&escrow_contract, &Symbol::new(&env, "lock_funds"), args);

        env.events().publish((Symbol::new(&env, "AgentSpend"),), (amount, token, seller));
        Ok(())
    }
}

