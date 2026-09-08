#![no_std]

pub mod errors;
pub mod types;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, Address, Env};
use crate::types::DataKey;
use crate::errors::TreasuryError;

const TTL_EXTEND: u32 = 535680;

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
            panic!("unauthorized");
        }

        env.storage().instance().set(&DataKey::DailyLimit, &new_limit);
        env.storage().instance().extend_ttl(TTL_EXTEND, TTL_EXTEND);
        
        env.events().publish((soroban_sdk::Symbol::new(&env, "LimitUpdated"),), new_limit);
        Ok(())
    }
}

