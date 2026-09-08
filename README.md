# VoxTrade: Autonomous x402 Machine-to-Machine Commerce

![CI Status](https://github.com/voxxtrade/voxtrade-contract/actions/workflows/ci.yml/badge.svg)
![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)
![Soroban SDK](https://img.shields.io/badge/Soroban-v20.0.0-orange)
![Target](https://img.shields.io/badge/Target-wasm32--unknown--unknown-lightgrey)

VoxTrade is an industry-grade Soroban smart contract architecture enabling sovereign, machine-to-machine (M2M) commerce on the Stellar network. It provides a secure execution layer for AI Voice Agents to negotiate B2B supply orders and settle payments using the x402 (Metered Payment Protocol) standard.

## ??? Architecture

The workspace is composed of two highly optimized, interdependent smart contracts:

1. **`agent_treasury`**: A non-custodial Smart Account controlled by the human merchant via Freighter, which enforces a mathematically sliding 24-hour spending limit on the AI's programmatic Ed25519 signing key.
2. **`x402_escrow`**: A specialized Hashed Time-Locked Contract (HTLC) that escrows funds pending cryptographic proof of delivery (the x402 macaroon preimage). Includes strict time-bound refund logic to prevent fund locking if the counterparty fails to deliver.

## ?? Quick Start

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (Edition 2021)
- `wasm32-unknown-unknown` target
- Soroban CLI (`cargo install --locked soroban-cli`)

### Build & Test
We provide a comprehensive `Makefile` to streamline development:

```bash
# Build optimized WASM binaries
make build

# Run unit tests and boundary condition checks
make test

# Format code and run clippy linters
make lint
```

## ?? Security & TTL Management
This repository strictly adheres to Soroban's State Archival guidelines. Instance and persistent storage TTLs are explicitly extended on every mutation to guarantee continuous availability on the Stellar ledger. All math utilizes safe `i128` operations to prevent overflow vulnerabilities.

## ?? Contributing
Please review our [CONTRIBUTING.md](./CONTRIBUTING.md) for strict Git workflow rules, including conventional commits and isolated branch strategies.

## ?? License
This project is licensed under the Apache 2.0 License - see the [LICENSE](./LICENSE) file for details.

