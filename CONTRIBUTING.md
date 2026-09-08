# Contributing to VoxTrade Contracts

We operate under a strict, disciplined Git workflow to maintain the integrity of our Soroban codebase.

## Workflow Rules
1. **Never use `git add .`**. Always stage files specifically related to the logical commit.
2. **One logical unit per commit.** (e.g., one function, one test block).
3. **Conventional Commits.** All commit messages must follow the `type(scope): description` format.
   - `feat(treasury): ...`
   - `fix(escrow): ...`
   - `test(treasury): ...`
   - `chore(workspace): ...`
4. **No Panics.** All logic must handle errors gracefully using `Result` and `#[contracterror]` enums.
5. **No `unwrap()` outside tests.**

