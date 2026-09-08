<div align="center">
  <h1>voxtrade-contract</h1>
  <p><strong>Sovereign Voice-to-Voice Commerce: Machine-to-Machine x402 Negotiation on Stellar</strong></p>
  <p>
    <img src="https://img.shields.io/github/actions/workflow/status/voxxtrade/voxtrade-contract/ci.yml?branch=main" alt="CI Status" />
    <img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License" />
    <img src="https://img.shields.io/badge/soroban--sdk-20.0.0--rc2-orange.svg" alt="soroban-sdk" />
  </p>
  <p>
    <a href="https://github.com/voxxtrade/voxtrade-app"><strong>VoxTrade App</strong></a> ·
    <a href="#getting-started"><strong>Getting Started</strong></a> ·
    <a href="#architecture"><strong>Architecture</strong></a>
  </p>
</div>

> Part of the **[VoxTrade](https://github.com/voxxtrade)** suite. This repository houses the Soroban smart contracts for Sovereign Voice-to-Voice Commerce, enabling secure, trustless M2M negotiation.

## The Problem

AI agents today can talk, but they can't transact. When an AI personal assistant negotiates on your behalf to purchase a service (like booking a flight or paying for a premium API via x402), it lacks a secure, bounded financial mechanism to execute the payment without exposing the user to infinite risk. 

Giving an AI direct access to a credit card or unrestricted wallet is a massive security vulnerability. Furthermore, if the AI pays up front and the counterparty fails to deliver the promised digital good or service, the money is lost.

## The Solution

VoxTrade bridges machine-to-machine x402 negotiation to the Stellar network using two purpose-built Soroban smart contracts:

1. **AgentTreasury** — A policy-bounded vault that grants an AI agent restricted spending power. It enforces a strict rolling daily limit. If the AI goes rogue or gets compromised, the maximum loss is mathematically capped.
2. **X402Escrow** — A trustless Hash Time-Locked Contract (HTLC) designed for x402 exchanges. Funds are locked on-chain and only released when the seller reveals the cryptographic preimage (the receipt/service key). If the seller fails to deliver within the timeout, the funds are refunded to the buyer.

## Enforced Invariants & Test Coverage

VoxTrade enforces strict invariants to guarantee safety in autonomous M2M transactions:

- **Strict Daily Quotas** — An agent can never exceed its daily_limit within a 24-hour ledger window (LimitExceeded).
- **Cryptographic Delivery Verification** — The escrow will only release funds if the provided preimage exactly matches the hash_lock established during negotiation (HashMismatch).
- **Guaranteed Refunds on Timeout** — If a seller fails to deliver the service preimage before the 	imeout_ledger, the buyer is guaranteed a full refund (TimeoutReached / TimeoutNotReached).
- **Zero Double-Spending** — An escrow can only be claimed or refunded exactly once. Once esolved, all subsequent interactions are blocked (AlreadyResolved).
- **Admin Isolation** — Only the administrative address can initialize the treasury or upgrade the configuration. The agent key is strictly sandboxed to spending (Unauthorized).

## Smart Contract Errors

All operational errors return explicit, stable integer codes.

### AgentTreasury (TreasuryError)

| Code | Variant | Description |
|---|---|---|
| 1 | Unauthorized | An action was attempted without the required admin or agent signature. |
| 2 | InsufficientBalance | The treasury lacks the underlying tokens to fund the requested escrow. |
| 3 | InvalidAmount | The requested amount is zero or negative. |
| 4 | NotFound | The requested configuration or daily spend record does not exist. |
| 5 | LimitExceeded | The requested transaction would exceed the agent's strict daily quota. |
| 6 | AlreadyInitialized | An attempt was made to re-initialize an already configured treasury. |

### X402Escrow (EscrowError)

| Code | Variant | Description |
|---|---|---|
| 1 | InvalidHash | The provided hash is structurally invalid. |
| 2 | AlreadyResolved | The escrow has already been successfully claimed or refunded. |
| 3 | TimeoutNotReached | A refund was requested before the designated timeout ledger. |
| 4 | TimeoutReached | A claim was attempted after the escrow had expired. |
| 5 | HashMismatch | The provided preimage does not produce the expected hash lock. |
| 6 | NotFound | The requested escrow ID does not exist in storage. |
| 7 | InvalidTimeout | The provided timeout ledger is in the past. |
| 8 | InvalidAmount | The escrow lock amount is zero or negative. |

## Architecture

VoxTrade operates on a commit-reveal scheme integrated with the x402 protocol:

`	ext
   User's AI Agent                  Service Provider (Seller)
         ¦                                   ¦
         +- 1. Negotiate Price & Terms -----?¦
         ¦?- 2. Provide Hash Lock (H) -------¦
         ¦                                   ¦
         ¦  (AgentTreasury validates quota)  ¦
         ¦                                   ¦
         +- 3. Lock Funds in X402Escrow ----?¦
         ¦                                   ¦
         ¦?- 4. Deliver Service + Preimage --¦
         ¦                                   ¦
         +- 5. Escrow.claim(Preimage)        ¦
         ¦  (Seller receives funds)          ¦
         ?                                   ?
`

1. The AI Agent negotiates with the seller.
2. The seller generates a random preimage and sends its SHA-256 hash_lock to the agent.
3. The agent calls AgentTreasury::execute_x402_lock(), which verifies the daily limit and forwards the funds to X402Escrow, locked under hash_lock.
4. The seller delivers the service along with the preimage.
5. The seller (or any party) submits the preimage to the escrow to release the funds. If they fail, the agent can call efund() after the timeout.

## Getting Started

### Prerequisites

Ensure you have the Rust toolchain and the Soroban CLI installed:

`ash
rustup target add wasm32-unknown-unknown
cargo install --locked soroban-cli
`

### Build and Test

`ash
# Run all unit tests
cargo test

# Build WASM artifacts for deployment
cargo build --target wasm32-unknown-unknown --release
`

## Testing

VoxTrade maintains a rigorous testing pipeline. Tests run against the Soroban test environment on every push. Our CI enforces:
- cargo fmt --check
- cargo clippy -D warnings
- cargo test

We simulate ledger time advancement to guarantee rolling limits reset accurately and timeout bounds function precisely as expected.

## License

MIT License. See [LICENSE](LICENSE) for details.
