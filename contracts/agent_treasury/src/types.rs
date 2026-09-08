use soroban_sdk::contracttype;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,      // Address
    AgentKey,   // Address
    DailyLimit, // i128
    SpentToday, // i128
    LastReset,  // u64
}
