use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    AgentKey,
    DailyLimit,
    SpentToday,
    LastReset,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub admin: Address,
    pub agent_key: Address,
    pub daily_limit: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DailySpend {
    pub day: u32,
    pub amount_spent: i128,
}
