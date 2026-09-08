use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum EscrowError {
    InvalidHash = 1,
    AlreadyResolved = 2,
    TimeoutNotReached = 3,
    TimeoutReached = 4,
}

