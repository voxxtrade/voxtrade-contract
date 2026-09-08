# Contract Reference

## Agent Treasury
- `init(env, admin, agent_key, daily_limit)`
- `update_limit(env, admin, new_limit)`
- `execute_x402_lock(env, agent, token, escrow, seller, amount, hash_lock, timeout)`

## x402 Escrow
- `lock_funds(env, buyer, seller, token, amount, hash_lock, timeout) -> u64`
- `claim(env, escrow_id, preimage)`
- `refund(env, escrow_id)`

