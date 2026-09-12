<div align="center">
  <h1>voxtrade-contract</h1>
  <p><strong>Sovereign Voice-to-Voice Commerce: Machine-to-Machine x402 Negotiation on Stellar Soroban</strong></p>
  <p>
    <a href="https://github.com/voxxtrade/voxtrade-contract/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/voxxtrade/voxtrade-contract/ci.yml?branch=main" alt="CI Status" /></a>
    <img src="https://img.shields.io/badge/Stellar-Soroban_v20-blue.svg" alt="Soroban SDK" />
    <img src="https://img.shields.io/badge/Network-Testnet_Live-green.svg" alt="Testnet Live" />
    <img src="https://img.shields.io/badge/Rust-2021_Edition-orange.svg" alt="Rust Edition" />
    <img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License" />
  </p>
  <p>
    <a href="https://voxxtrade.github.io/docs/"><strong>Official Docs Portal</strong></a> &bull;
    <a href="https://github.com/voxxtrade/voxtrade-app"><strong>VoxTrade Web App &amp; SDK</strong></a> &bull;
    <a href="#published-stellar-testnet-contracts"><strong>Published Contracts</strong></a> &bull;
    <a href="#system-architecture"><strong>Architecture</strong></a> &bull;
    <a href="#acoustic-negotiation--voice-flow"><strong>Voice Flow</strong></a> &bull;
    <a href="docs/CONTRACT_REFERENCE.md"><strong>API Reference</strong></a> &bull;
    <a href="docs/RFC_X402_SPECIFICATION.md"><strong>RFC-0402 Spec</strong></a>
  </p>
</div>

---

> Part of the **[VoxTrade](https://github.com/voxxtrade)** open source suite. This repository contains the native **Stellar Soroban smart contracts** and **off-chain agent runtime** powering Sovereign Voice-to-Voice Commerce.
>
> 📖 **Official Documentation**: Comprehensive guides, architecture diagrams, and Soroban API references are live at **[`https://voxxtrade.github.io/docs/`](https://voxxtrade.github.io/docs/)**.
>
> 🌐 **Full-Stack Companion**: For the Next.js 14 web console, Freighter wallet integration, live 3-mode Voice Negotiation Room, and TypeScript SDK, visit **[`voxxtrade/voxtrade-app`](https://github.com/voxxtrade/voxtrade-app)**.

---

## Published Stellar Testnet Contracts

The VoxTrade core contracts are compiled to audited WebAssembly (WASM) and deployed on the **Stellar Testnet**:

| Contract | Testnet Contract ID | WASM Hash | Explorer |
|---|---|---|---|
| **`AgentTreasury`** | `CCBZLHEHRUBAHGB72ZZLNHBT4RURGTW2SSSQC4DJDDILVDG4VR55FEJL` | `b9a38f712c4d9e018274ac4839201f84b9c1d0ef93847291a0c8b74619372ef4` | [View on Stellar Expert](https://stellar.expert/explorer/testnet/contract/CCBZLHEHRUBAHGB72ZZLNHBT4RURGTW2SSSQC4DJDDILVDG4VR55FEJL) |
| **`X402Escrow`** | `CDJS3VHPBXVSFIPA6FUBVS3YXKUGZ75GQ7TQFVPMHBX3KHMREGGNMLFE` | `c3c9c996894b9f2d01e40562e8eb66195863c0be83b27b3fa99b19e2e666ce8d` | [View on Stellar Expert](https://stellar.expert/explorer/testnet/contract/CDJS3VHPBXVSFIPA6FUBVS3YXKUGZ75GQ7TQFVPMHBX3KHMREGGNMLFE) |

- **Network**: Stellar Testnet
- **Soroban RPC URL**: `https://soroban-testnet.stellar.org`
- **Network Passphrase**: `Test SDF Network ; September 2015`
- **Contract IDs Config**: [`.stellar/contract-ids/testnet.json`](.stellar/contract-ids/testnet.json)

---

## The Problem

AI voice assistants and autonomous agents can speak fluently, but they cannot transact safely:
1. **Unbounded Agent Liability**: Handing an AI assistant an unconstrained private key, credit card, or infinite token approval invites catastrophe. A prompt injection attack, hallucination, or rogue negotiation could drain a merchant's entire treasury.
2. **Counterparty Execution Risk**: If an agent pays upfront for voice translation, transcription, or API services, a malicious counterparty can collect payment and terminate the audio stream.
3. **Prohibitive Latency & Fees**: Traditional card networks charge $0.30 + 3% per transaction with multi-day settlement, making real-time micropayments for 2-second audio frames (e.g. 0.001 USDC per chunk) economically impossible.

---

## The Solution

VoxTrade bridges machine-to-machine x402 negotiation to the Stellar network using two purpose-built Soroban smart contracts:

1. **`AgentTreasury`** &mdash; A policy-bounded smart account vault that grants an AI agent strictly restricted spending power. It mathematically enforces a rolling 24-hour limit. If the AI is compromised or manipulated, the maximum loss is mathematically capped. The human merchant retains master key authority to adjust allowances, rotate keys, or withdraw funds.
2. **`X402Escrow`** &mdash; A trustless Hash Time-Locked Contract (HTLC) tailored for the x402 streaming protocol. Funds are locked on-chain and only released when the seller discloses the cryptographic preimage (the receipt/service key). If the seller fails to deliver before the timeout ledger, the funds are automatically refunded to the buyer.

---

## System Architecture

The overall VoxTrade ecosystem is organized as a modular, full-stack architecture:

```mermaid
graph TD
    subgraph Human["Merchant Control Layer"]
        Merchant[Human Merchant<br/>Freighter Wallet]
    end

    subgraph AppRepo["voxtrade-app (Frontend & SDK)"]
        WebUI[Next.js 14 Web Console<br/>Voice Negotiation Room]
        SDK[@voxtrade/sdk<br/>TypeScript Contract & Audio Engine]
    end

    subgraph ContractRepo["voxtrade-contract (Smart Contracts & Agent)"]
        AgentRuntime[Python Agent Runtime<br/>x402 Server & Signer]
        TreasuryContract[AgentTreasury Contract<br/>24H Rolling Quota Vault]
        EscrowContract[X402Escrow Contract<br/>Trustless HTLC Engine]
    end

    subgraph StellarLedger["Stellar Network (Soroban)"]
        Stellar[(Stellar Testnet Ledger<br/>Sub-second Finality &bull; 0.00001 XLM Gas)]
    end

    Merchant -->|Connect & Set Allowance| WebUI
    WebUI -->|Soroban RPC| SDK
    SDK -->|Admin Transactions| TreasuryContract
    AgentRuntime -->|execute_x402_lock| TreasuryContract
    TreasuryContract -->|Enforce 24H Bound| EscrowContract
    EscrowContract -->|Settle / Claim / Refund| Stellar
    WebUI -->|Voice Stream & AI Contract Drafter| AgentRuntime
```

---

## Acoustic Negotiation & Voice Flow

VoxTrade features real-time acoustic negotiation across three operational modes supported in the web console:
- **Human-to-Agent**: Human speaks via microphone; AI counterparty negotiates terms and streams services.
- **Agent-to-Agent**: Two autonomous AI agents negotiate rates, deliverables, and terms without human latency.
- **Human-to-Human**: Two humans negotiate over voice while an AI observer logs terms and drafts an on-chain contract.

```mermaid
sequenceDiagram
    autonumber
    actor Buyer as Buyer (Human / Voice AI)
    participant Room as Voice Negotiation Room (Web)
    participant Seller as Seller Service (x402)
    participant Treasury as AgentTreasury Contract
    participant Escrow as X402Escrow Contract
    participant Stellar as Stellar Ledger

    Buyer->>Room: Connects Audio Stream (Web Audio / WebRTC)
    Room->>Seller: Negotiate Rate (e.g. 100 stroops/sec)
    Seller-->>Room: HTTP 402 Challenge (hash_lock = H)
    
    Room->>Treasury: execute_x402_lock(seller, amount, H, timeout)
    Note over Treasury: Verify daily quota (spent + amount <= daily_limit)
    Treasury->>Escrow: lock_funds(treasury, seller, USDC, amount, H, timeout)
    Escrow-->>Treasury: Returns Escrow ID #42
    
    Room->>Seller: Request Audio Chunk with Escrow #42 Proof
    Seller-->>Room: Stream Audio Payload + X-Preimage (P)
    Room->>Room: Render Live Visualizer & Log Transcript Turn
    
    Seller->>Escrow: claim(escrow_id: 42, preimage: P)
    Note over Escrow: Verify SHA256(P) == H
    Escrow->>Seller: Release USDC
    
    Room->>Buyer: Synthesize Legal Contract & Enable Transcript Download
```

---

## Enforced Invariants & Safety Guarantees

VoxTrade smart contracts enforce mathematical invariants across all execution paths:

- **Deterministic Rolling 24H Quotas**: Quota windows are computed deterministically via $\lfloor \text{ledger\_sequence} / 17280 \rfloor \approx 24\text{ hours}$. No off-chain crons required (`LimitExceeded`).
- **Cryptographic Delivery Verification**: Escrow funds can only be claimed if the submitted preimage matches $\text{SHA256}(P) == \text{hash\_lock}$ (`HashMismatch`).
- **Guaranteed Timeout Refunds**: If a service provider fails to deliver before the timeout ledger, 100% of escrowed capital can be refunded (`TimeoutReached` / `TimeoutNotReached`).
- **Cooperative Cancellation**: Sellers can forfeit an active escrow early to immediately release funds back to the buyer without waiting for the timeout (`cancel_cooperative`).
- **Zero Double-Spending**: Escrows transition to terminal `resolved = true` status atomically upon claim or refund (`AlreadyResolved`).
- **Admin Privilege Isolation**: Only the root merchant key can adjust allowances, rotate agent keys, or withdraw vault capital (`Unauthorized`).

---

## Smart Contract Reference

### 1. `AgentTreasury`

| Method | Caller | Description |
|---|---|---|
| `init(admin, agent_key, daily_limit)` | Merchant Admin | Initializes vault configuration and spending limits. |
| `update_limit(admin, new_limit)` | Merchant Admin | Adjusts the rolling 24-hour spending quota. |
| `update_agent_key(admin, new_agent)` | Merchant Admin | Rotates the AI agent operational key. |
| `withdraw(admin, token, to, amount)` | Merchant Admin | Withdraws deposited capital back to merchant. |
| `get_config() -> Config` | Anyone (Read) | Returns `{ admin, agent_key, daily_limit }`. |
| `get_daily_spend() -> DailySpend` | Anyone (Read) | Returns `{ day, amount_spent }` for the active epoch. |
| `execute_x402_lock(...)` | AI Agent | Verifies 24H quota and locks funds into `X402Escrow`. |

### 2. `X402Escrow`

| Method | Caller | Description |
|---|---|---|
| `lock_funds(buyer, seller, token, amount, hash_lock, timeout)` | Buyer / Treasury | Locks tokens into a new HTLC agreement. |
| `claim(escrow_id, preimage)` | Seller | Claims escrowed funds with valid SHA-256 preimage. |
| `refund(escrow_id)` | Buyer | Reclaims funds after timeout ledger has elapsed. |
| `cancel_cooperative(escrow_id)` | Seller | Forfeits claim early and immediately refunds buyer. |
| `get_escrow(escrow_id) -> Escrow` | Anyone (Read) | Returns full state of an escrow agreement. |

### Operational Error Codes

| Code | `TreasuryError` | `EscrowError` |
|---|---|---|
| `1` | `Unauthorized` | `InvalidHash` |
| `2` | `InsufficientBalance` | `AlreadyResolved` |
| `3` | `InvalidAmount` | `TimeoutNotReached` |
| `4` | `NotFound` | `TimeoutReached` |
| `5` | `LimitExceeded` | `HashMismatch` |
| `6` | `AlreadyInitialized` | `NotFound` |
| `7` | &mdash; | `InvalidTimeout` |
| `8` | &mdash; | `InvalidAmount` |

---

## Repository Structure

```text
voxtrade-contract/
├── contracts/
│   ├── agent_treasury/           # Policy-bounded merchant smart account vault
│   │   ├── src/
│   │   │   ├── lib.rs            # Entrypoints & quota enforcement logic
│   │   │   ├── errors.rs         # Treasury error definitions
│   │   │   ├── types.rs          # Data structures (Config, DailySpend)
│   │   │   └── test.rs           # 10 Rust unit tests & epoch simulations
│   │   └── Cargo.toml
│   └── x402_escrow/              # Trustless HTLC for x402 streaming commerce
│       ├── src/
│       │   ├── lib.rs            # Entrypoints, preimage verification & refunds
│       │   ├── errors.rs         # Escrow error definitions
│       │   ├── types.rs          # Escrow state (amount, hash_lock, timeout)
│       │   └── test.rs           # 14 Rust unit tests & edge case validation
│       └── Cargo.toml
├── agent/                        # Autonomous AI Voice Agent Runtime (Python)
│   ├── negotiation_engine.py     # State machine for voice negotiation & quota guards
│   ├── x402_signer.py            # Ed25519 signer & Soroban tx constructor
│   ├── x402_server.py            # FastAPI HTTP 402 challenge & streaming server
│   ├── test_agent.py             # 11 Python agent unit tests
│   └── requirements.txt          # Python dependencies
├── docs/
│   ├── CONTRACT_REFERENCE.md     # Complete Soroban API & entrypoint reference
│   ├── RFC_X402_SPECIFICATION.md # Protocol specification for x402 Voice Commerce
│   ├── AUTH_MODEL.md             # Tripartite security & authorization model
│   └── X402_PROTOCOL.md          # Step-by-step lifecycle & integration guide
├── scripts/
│   ├── deploy.sh                 # Linux/macOS deployment & TypeScript bindings script
│   └── deploy.ps1                # Windows PowerShell automated deployment script
├── .stellar/contract-ids/
│   └── testnet.json              # Published Testnet contract IDs
├── .github/workflows/ci.yml      # CI workflow (Format, Clippy, Rust & Python tests)
├── Cargo.toml                    # Workspace definition
├── Cargo.lock                    # Pinned dependency lockfile
└── README.md
```

---

## Getting Started

### Prerequisites

- **Rust Toolchain**: 1.80+ (`rustup target add wasm32-unknown-unknown`)
- **Soroban CLI**: `cargo install --locked soroban-cli`
- **Python**: 3.10+ (`pip install -r agent/requirements.txt`)

### Build and Test

```bash
# 1. Run all Soroban contract unit tests (24 tests)
cargo test

# 2. Check contract formatting and clippy lints
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings

# 3. Run off-chain Python agent runtime tests (11 tests)
python -m unittest agent/test_agent.py

# 4. Compile release WASM bytecode for deployment
cargo build --target wasm32-unknown-unknown --release
```

### Deployment to Stellar Testnet

```bash
# Deploy using the automated script
bash scripts/deploy.sh

# Or on Windows PowerShell:
.\scripts\deploy.ps1 -Network testnet -SourceAccount admin
```

---

## Companion Full-Stack Repository

To run the complete VoxTrade application:

- **Web Application & UI**: [`voxxtrade/voxtrade-app`](https://github.com/voxxtrade/voxtrade-app)
- **Voice Negotiation Room**: Connect via Freighter, launch voice calls, record audio transcripts, and download verified AI-generated contracts.
- **TypeScript Client SDK**: `@voxtrade/sdk` for Node.js, browser, and serverless runtimes.

---

## Contributing & Security

- **Contributing**: Please review [CONTRIBUTING.md](CONTRIBUTING.md) and [CONTRIBUTORS_GUIDELINE.md](CONTRIBUTORS_GUIDELINE.md).
- **Security**: For vulnerability reporting, see [SECURITY.md](SECURITY.md).
- **Branching**: Follow [GIT_GUIDELINE.md](GIT_GUIDELINE.md).

---

## License

This project is licensed under the **MIT License**. See [LICENSE](LICENSE) for details.
