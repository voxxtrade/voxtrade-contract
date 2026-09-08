#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, IntoVal};

mod errors;
mod types;
#[cfg(test)]
mod test;

use errors::TreasuryError;
use types::{Config, DailySpend};

#[contract]
pub struct AgentTreasury;

#[contractimpl]
impl AgentTreasury {
    pub fn init(env: Env, admin: Address, agent_key: Address, daily_limit: i128) -> Result<(), TreasuryError> {
        admin.require_auth();
        if env.storage().instance().has(&Symbol::new(&env, "admin")) {
            return Err(TreasuryError::AlreadyInitialized);
        }
        let config = Config { admin, agent_key, daily_limit };
        env.storage().instance().set(&Symbol::new(&env, "config"), &config);
        env.storage().instance().extend_ttl(100_000, 100_000);
        Ok(())
    }

    pub fn execute_x402_lock(
        env: Env,
        agent: Address,
        token: Address,
        escrow: Address,
        seller: Address,
        amount: i128,
        hash_lock: soroban_sdk::BytesN<32>,
        timeout_ledger: u32,
    ) -> Result<(), TreasuryError> {
        agent.require_auth();
        let config: Config = env.storage().instance().get(&Symbol::new(&env, "config")).unwrap();
        if agent != config.agent_key {
            return Err(TreasuryError::Unauthorized);
        }

        let current_ledger = env.ledger().sequence();
        let current_day = current_ledger / 17280;

        let spend_key = Symbol::new(&env, "daily_spend");
        let mut daily_spend: DailySpend = env.storage().persistent().get(&spend_key).unwrap_or(DailySpend {
            day: current_day,
            amount_spent: 0,
        });

        if daily_spend.day != current_day {
            daily_spend.day = current_day;
            daily_spend.amount_spent = 0;
        }

        if daily_spend.amount_spent + amount > config.daily_limit {
            return Err(TreasuryError::LimitExceeded);
        }

        daily_spend.amount_spent += amount;
        env.storage().persistent().set(&spend_key, &daily_spend);
        env.storage().persistent().extend_ttl(&spend_key, 100_000, 100_000);

        env.invoke_contract::<()>(
            &escrow,
            &Symbol::new(&env, "lock_funds"),
            (
                env.current_contract_address().into_val(&env),
                seller.into_val(&env),
                token.into_val(&env),
                amount.into_val(&env),
                hash_lock.into_val(&env),
                timeout_ledger.into_val(&env),
            ).into_val(&env),
        );

        Ok(())
    }
}
