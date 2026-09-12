# VoxTrade Soroban Smart Contract Reference

This document provides a comprehensive technical reference for the Soroban smart contracts powering **VoxTrade: Sovereign Voice-to-Voice Commerce on Stellar**.

---

## 1. Published Stellar Testnet Deployments

| Contract | Address | WASM Hash | Stellar Expert Explorer |
|---|---|---|---|
| **AgentTreasury** | `CCBZLHEHRUBAHGB72ZZLNHBT4RURGTW2SSSQC4DJDDILVDG4VR55FEJL` | `b9a38f712c4d9e018274ac4839201f84b9c1d0ef93847291a0c8b74619372ef4` | [View on Stellar Expert](https://stellar.expert/explorer/testnet/contract/CCBZLHEHRUBAHGB72ZZLNHBT4RURGTW2SSSQC4DJDDILVDG4VR55FEJL) |
| **X402Escrow** | `CDJS3VHPBXVSFIPA6FUBVS3YXKUGZ75GQ7TQFVPMHBX3KHMREGGNMLFE` | `c3c9c996894b9f2d01e40562e8eb66195863c0be83b27b3fa99b19e2e666ce8d` | [View on Stellar Expert](https://stellar.expert/explorer/testnet/contract/CDJS3VHPBXVSFIPA6FUBVS3YXKUGZ75GQ7TQFVPMHBX3KHMREGGNMLFE) |

- **Network**: Stellar Testnet
- **Network Passphrase**: `Test SDF Network ; September 2015`
- **Soroban RPC URL**: `https://soroban-testnet.stellar.org`

---

## 2. AgentTreasury Contract (`contracts/agent_treasury`)

The `AgentTreasury` contract acts as a cryptographically bounded spending vault for autonomous AI voice agents. The human merchant retains root administrative authority, while the AI is sandboxed to spend within a rolling 24-hour limit.

### 2.1 State & Storage Model

- **Instance Storage (`Symbol("config")`)**:
  - `admin: Address` &mdash; Root merchant account with administrative privileges.
  - `agent_key: Address` &mdash; Authorized AI agent signing key.
  - `daily_limit: i128` &mdash; Maximum allowance per 24-hour window (in token stroops).
- **Persistent Storage (`Symbol("daily_spend")`)**:
  - `day: u32` &mdash; Current 24-hour epoch calculated via `ledger_sequence / 17280`.
  - `amount_spent: i128` &mdash; Cumulative stroops spent during the active day epoch.

### 2.2 Public Entrypoints

#### `init(env: Env, admin: Address, agent_key: Address, daily_limit: i128) -> Result<(), TreasuryError>`
Initializes the treasury with root administrator credentials, agent signing address, and initial spending allowance.
- **Auth Required**: `admin.require_auth()`
- **Errors**: `AlreadyInitialized` (Code 6)

#### `update_limit(env: Env, admin: Address, new_limit: i128) -> Result<(), TreasuryError>`
Modifies the rolling 24-hour spending allowance.
- **Auth Required**: `admin.require_auth()`
- **Errors**: `InvalidAmount` (Code 3), `NotFound` (Code 4), `Unauthorized` (Code 1)

#### `update_agent_key(env: Env, admin: Address, new_agent_key: Address) -> Result<(), TreasuryError>`
Rotates the authorized agent signing address without needing to redeploy or migrate the treasury vault.
- **Auth Required**: `admin.require_auth()`
- **Errors**: `NotFound` (Code 4), `Unauthorized` (Code 1)

#### `withdraw(env: Env, admin: Address, token: Address, to: Address, amount: i128) -> Result<(), TreasuryError>`
Allows the merchant administrator to withdraw deposited tokens back to a designated account.
- **Auth Required**: `admin.require_auth()`
- **Errors**: `InvalidAmount` (Code 3), `NotFound` (Code 4), `Unauthorized` (Code 1)

#### `get_config(env: Env) -> Result<Config, TreasuryError>`
Read-only query returning the current treasury configuration (`admin`, `agent_key`, `daily_limit`).

#### `get_daily_spend(env: Env) -> DailySpend`
Read-only query returning the current day index (`ledger_sequence / 17280`) and cumulative `amount_spent`. Automatically resets to `0` when querying an expired epoch.

#### `execute_x402_lock(env: Env, agent: Address, token: Address, escrow: Address, seller: Address, amount: i128, hash_lock: BytesN<32>, timeout_ledger: u32) -> Result<(), TreasuryError>`
Invoked by the AI agent when acoustic negotiations conclude. Verifies daily quota compliance, records cumulative spend, and automatically calls `X402Escrow::lock_funds`.
- **Auth Required**: `agent.require_auth()`
- **Errors**: `Unauthorized` (Code 1), `LimitExceeded` (Code 5)

---

## 3. X402Escrow Contract (`contracts/x402_escrow`)

The `X402Escrow` contract implements a trustless Hash Time-Locked Contract (HTLC) tailored for machine-to-machine streaming and x402 payment requirements.

### 3.1 State & Storage Model

- **Persistent Storage (`DataKey::Nonce`)**:
  - Auto-incrementing `u64` identifying each unique escrow agreement.
- **Persistent Storage (`DataKey::Escrow(u64)`)**:
  - `buyer: Address` &mdash; Account or treasury funding the escrow.
  - `seller: Address` &mdash; Service provider or streaming counterparty.
  - `token: Address` &mdash; SAC / SEP-41 token address (e.g. USDC).
  - `amount: i128` &mdash; Escrowed amount in stroops.
  - `hash_lock: BytesN<32>` &mdash; SHA-256 digest of the negotiated secret preimage.
  - `timeout_ledger: u32` &mdash; Ledger sequence deadline before refund becomes available.
  - `resolved: bool` &mdash; Terminal flag preventing replay attacks or double-claims.

### 3.2 Public Entrypoints

#### `lock_funds(env: Env, buyer: Address, seller: Address, token: Address, amount: i128, hash_lock: BytesN<32>, timeout_ledger: u32) -> Result<u64, EscrowError>`
Transfers tokens from `buyer` into escrow custody and registers a new HTLC agreement.
- **Auth Required**: `buyer.require_auth()`
- **Returns**: Unique `u64` escrow ID.
- **Errors**: `InvalidAmount` (Code 8), `InvalidTimeout` (Code 7)

#### `claim(env: Env, escrow_id: u64, preimage: BytesN<32>) -> Result<(), EscrowError>`
Claimed by the seller by revealing the 32-byte secret preimage. The contract computes `SHA256(preimage)` on-chain and validates equality against `hash_lock`. Upon match, funds are released to `seller`.
- **Errors**: `NotFound` (Code 6), `AlreadyResolved` (Code 2), `TimeoutReached` (Code 4), `HashMismatch` (Code 5)

#### `refund(env: Env, escrow_id: u64) -> Result<(), EscrowError>`
Allows the buyer to reclaim locked funds after the designated `timeout_ledger` has elapsed if the seller failed to deliver the preimage.
- **Errors**: `NotFound` (Code 6), `AlreadyResolved` (Code 2), `TimeoutNotReached` (Code 3)

#### `cancel_cooperative(env: Env, escrow_id: u64) -> Result<(), EscrowError>`
Allows the seller to forfeit their claim early and immediately refund the buyer before the timeout expires (e.g. in case of call disconnection or service abort).
- **Auth Required**: `seller.require_auth()`
- **Errors**: `NotFound` (Code 6), `AlreadyResolved` (Code 2)

#### `get_escrow(env: Env, escrow_id: u64) -> Result<Escrow, EscrowError>`
Read-only query returning the complete state of an escrow agreement.

---

## 4. Operational Error Reference

### 4.1 Treasury Errors (`TreasuryError`)

| Integer Code | Enum Variant | Cause |
|---|---|---|
| `1` | `Unauthorized` | Caller lacks required admin or agent signature. |
| `2` | `InsufficientBalance` | Vault has insufficient token balance for transfer. |
| `3` | `InvalidAmount` | Requested limit or transfer amount is $\le 0$. |
| `4` | `NotFound` | Treasury configuration record not initialized. |
| `5` | `LimitExceeded` | Transaction would breach the 24-hour daily quota. |
| `6` | `AlreadyInitialized` | Treasury has already been initialized. |

### 4.2 Escrow Errors (`EscrowError`)

| Integer Code | Enum Variant | Cause |
|---|---|---|
| `1` | `InvalidHash` | Hash structure is malformed. |
| `2` | `AlreadyResolved` | Escrow has already been claimed or refunded. |
| `3` | `TimeoutNotReached` | Refund requested before timeout ledger. |
| `4` | `TimeoutReached` | Claim attempted after timeout ledger elapsed. |
| `5` | `HashMismatch` | `SHA256(preimage) != hash_lock`. |
| `6` | `NotFound` | Specified escrow ID does not exist. |
| `7` | `InvalidTimeout` | Timeout ledger is set in the past. |
| `8` | `InvalidAmount` | Escrow amount is $\le 0$. |

---

## 5. Published Contract Events

### AgentTreasury Events
- `("treasury", "init")`: `(admin, agent_key, daily_limit)`
- `("treasury", "update_limit")`: `(admin, old_limit, new_limit)`
- `("treasury", "update_agent_key")`: `(admin, old_agent, new_agent)`
- `("treasury", "withdraw")`: `(admin, to, token, amount)`
- `("treasury", "x402_lock")`: `(agent, escrow, seller, amount)`

### X402Escrow Events
- `("escrow", "lock_funds")`: `(escrow_id, buyer, seller, amount, hash_lock, timeout_ledger)`
- `("escrow", "claim")`: `(escrow_id, seller, amount, preimage)`
- `("escrow", "refund")`: `(escrow_id, buyer, amount)`
- `("escrow", "cancel")`: `(escrow_id, buyer, seller, amount)`


