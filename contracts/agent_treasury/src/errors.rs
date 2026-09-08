use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum TreasuryError {
    Unauthorized = 1,
    InsufficientBalance = 2,
    InvalidAmount = 3,
    NotFound = 4,
    LimitExceeded = 5,
}
