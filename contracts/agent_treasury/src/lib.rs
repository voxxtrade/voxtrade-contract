#![no_std]
#![allow(clippy::too_many_arguments)]
use soroban_sdk::{contract, contractimpl, Address, Env, IntoVal, Symbol};

mod errors;
#[cfg(test)]
mod test;
mod types;

use errors::TreasuryError;
use types::{Config, DailySpend};

#[contract]
pub struct AgentTreasury;

#[contractimpl]
#[allow(clippy::too_many_arguments)]
impl AgentTreasury {
    pub fn init(
        env: Env,
        admin: Address,
        agent_key: Address,
        daily_limit: i128,
    ) -> Result<(), TreasuryError> {
        admin.require_auth();
        if env.storage().instance().has(&Symbol::new(&env, "config")) {
            return Err(TreasuryError::AlreadyInitialized);
        }
        let config = Config {
            admin: admin.clone(),
            agent_key: agent_key.clone(),
            daily_limit,
        };
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "config"), &config);
        env.events().publish(
            (Symbol::new(&env, "treasury"), Symbol::new(&env, "init")),
            (admin, agent_key, daily_limit),
        );
        Ok(())
    }

    pub fn update_limit(env: Env, admin: Address, new_limit: i128) -> Result<(), TreasuryError> {
        admin.require_auth();
        if new_limit <= 0 {
            return Err(TreasuryError::InvalidAmount);
        }
        let mut config: Config = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "config"))
            .ok_or(TreasuryError::NotFound)?;
        if admin != config.admin {
            return Err(TreasuryError::Unauthorized);
        }
        let old_limit = config.daily_limit;
        config.daily_limit = new_limit;
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "config"), &config);
        env.storage().instance().extend_ttl(100_000, 100_000);

        env.events().publish(
            (
                Symbol::new(&env, "treasury"),
                Symbol::new(&env, "update_limit"),
            ),
            (admin, old_limit, new_limit),
        );
        Ok(())
    }

    pub fn update_agent_key(
        env: Env,
        admin: Address,
        new_agent_key: Address,
    ) -> Result<(), TreasuryError> {
        admin.require_auth();
        let mut config: Config = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "config"))
            .ok_or(TreasuryError::NotFound)?;
        if admin != config.admin {
            return Err(TreasuryError::Unauthorized);
        }
        let old_agent = config.agent_key.clone();
        config.agent_key = new_agent_key.clone();
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "config"), &config);
        env.storage().instance().extend_ttl(100_000, 100_000);

        env.events().publish(
            (
                Symbol::new(&env, "treasury"),
                Symbol::new(&env, "update_agent_key"),
            ),
            (admin, old_agent, new_agent_key),
        );
        Ok(())
    }

    pub fn withdraw(
        env: Env,
        admin: Address,
        token: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), TreasuryError> {
        admin.require_auth();
        if amount <= 0 {
            return Err(TreasuryError::InvalidAmount);
        }
        let config: Config = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "config"))
            .ok_or(TreasuryError::NotFound)?;
        if admin != config.admin {
            return Err(TreasuryError::Unauthorized);
        }

        soroban_sdk::token::Client::new(&env, &token).transfer(
            &env.current_contract_address(),
            &to,
            &amount,
        );

        env.events().publish(
            (Symbol::new(&env, "treasury"), Symbol::new(&env, "withdraw")),
            (admin, to, token, amount),
        );
        Ok(())
    }

    pub fn get_config(env: Env) -> Result<Config, TreasuryError> {
        env.storage()
            .instance()
            .get(&Symbol::new(&env, "config"))
            .ok_or(TreasuryError::NotFound)
    }

    pub fn get_daily_spend(env: Env) -> DailySpend {
        let current_ledger = env.ledger().sequence();
        let current_day = current_ledger / 17280;
        let spend_key = Symbol::new(&env, "daily_spend");
        let daily_spend: DailySpend =
            env.storage()
                .persistent()
                .get(&spend_key)
                .unwrap_or(DailySpend {
                    day: current_day,
                    amount_spent: 0,
                });

        if daily_spend.day != current_day {
            DailySpend {
                day: current_day,
                amount_spent: 0,
            }
        } else {
            daily_spend
        }
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
        let config: Config = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "config"))
            .unwrap();
        if agent != config.agent_key {
            return Err(TreasuryError::Unauthorized);
        }

        let current_ledger = env.ledger().sequence();
        let current_day = current_ledger / 17280;

        let spend_key = Symbol::new(&env, "daily_spend");
        let mut daily_spend: DailySpend =
            env.storage()
                .persistent()
                .get(&spend_key)
                .unwrap_or(DailySpend {
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
        env.storage()
            .persistent()
            .extend_ttl(&spend_key, 100_000, 100_000);

        env.invoke_contract::<u64>(
            &escrow,
            &Symbol::new(&env, "lock_funds"),
            soroban_sdk::vec![
                &env,
                env.current_contract_address().into_val(&env),
                seller.into_val(&env),
                token.into_val(&env),
                amount.into_val(&env),
                hash_lock.into_val(&env),
                timeout_ledger.into_val(&env),
            ],
        );

        env.events().publish(
            (
                Symbol::new(&env, "treasury"),
                Symbol::new(&env, "x402_lock"),
            ),
            (agent, escrow, seller, amount),
        );

        Ok(())
    }
}
