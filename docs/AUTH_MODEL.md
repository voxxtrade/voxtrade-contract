# Smart Account Authorization Model

The `agent_treasury` relies on Stellar's native `require_auth()` capability.
- **Admin (Freighter):** The human merchant holds the master key and signs transactions to increase/decrease the AI's daily spending limit.
- **Agent (Ed25519 Backend Key):** The Voice AI server holds a standard signing key. It can *only* invoke the `execute_x402_lock` function, and the contract mathematically reverts if the limit is breached.

