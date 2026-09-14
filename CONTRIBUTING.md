# Contributing to VoxTrade Contracts

Thank you for your interest in contributing to **VoxTrade Soroban Smart Contracts**! We welcome community improvements, gas optimizations, and security hardening.

---

## Code of Conduct

All contributors are expected to uphold our [Code of Conduct](CODE_OF_CONDUCT.md). Please treat all community members with respect and professionalism.

---

## How to Get Started

1. **Browse Open Issues**: Check [GitHub Issues](https://github.com/voxxtrade/voxtrade-contract/issues) for bug reports, task enhancements, or open discussions.
2. **Coordinate First**: Comment on the issue you wish to work on to avoid duplicate work and discuss implementation strategies with the maintainers.
3. **Submit a PR**: Once ready, open a Pull Request adhering to the quality checklist below.

---

## Local Development Workflow

### 1. Prerequisites

- **Rust**: `v1.80+` (via `rustup`)
- **WASM Target**: `rustup target add wasm32-unknown-unknown`
- **Stellar CLI**: `v21+` (`cargo install --locked stellar-cli --features opt`)

### 2. Fork & Clone

```bash
git clone https://github.com/<your-username>/voxtrade-contract.git
cd voxtrade-contract
```

### 3. Build & Test

```bash
# Run all contract unit and integration tests
cargo test

# Build optimized WASM binaries
stellar contract build
```

### 4. Create a Working Branch

```bash
git checkout -b feat/<short-description>
# or
git checkout -b fix/<short-description>
```

---

## Pre-Flight Quality Checklist

Before submitting your PR, ensure the following commands run cleanly:

```bash
# 1. Format code
cargo fmt --check

# 2. Clippy linting
cargo clippy --all-targets -- -D warnings

# 3. Test suite
cargo test
```

---

## Commit & PR Guidelines

- Follow **Conventional Commits**: `feat:`, `fix:`, `docs:`, `test:`, `refactor:`.
- Reference related issues in your PR description: `Closes #<NUMBER>`.
- Thoroughly document any gas, storage footprint, or authorization changes in your PR description.
