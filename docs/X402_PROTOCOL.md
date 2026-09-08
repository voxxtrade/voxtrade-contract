# x402 Protocol Implementation

The `x402_escrow` contract implements the Lightning Network's L402 protocol adapted for Stellar smart contracts.
It uses a Hashed Time-Locked Contract (HTLC) to ensure the Voice AI does not release funds until cryptographic proof of delivery is provided.

## Sequence Diagram
1. Merchant AI negotiates price.
2. Merchant AI generates `preimage` and `hash_lock = SHA256(preimage)`.
3. Merchant AI calls `agent_treasury::execute_x402_lock`.
4. Supplier AI provides service.
5. Merchant AI gives `preimage` to Supplier AI.
6. Supplier AI calls `x402_escrow::claim(preimage)`.

