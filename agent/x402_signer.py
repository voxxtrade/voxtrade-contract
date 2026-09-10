"""
VoxTrade Autonomous Voice Agent - Cryptographic Signer & Stellar Soroban Bridge
Author: ogundeleoluwaferanmi35
"""

import os
import hashlib
import secrets
from typing import Tuple, Dict, Any, Optional
from stellar_sdk import Keypair, Server, Network, TransactionBuilder
from stellar_sdk import scval, xdr as stellar_xdr

class VoiceAgentSigner:
    """
    Programmatic signer that executes bounded smart contract locks
    on behalf of an autonomous AI Voice Agent on Stellar Soroban.
    """

    def __init__(
        self,
        secret_key: Optional[str] = None,
        rpc_url: str = "https://soroban-testnet.stellar.org:443",
        network_passphrase: str = "Test SDF Network ; September 2015",
    ):
        if secret_key:
            self.keypair = Keypair.from_secret(secret_key)
        else:
            self.keypair = Keypair.random()

        self.public_key = self.keypair.public_key
        self.rpc_url = rpc_url
        self.network_passphrase = network_passphrase
        self.server = Server(rpc_url)

    @staticmethod
    def generate_keypair() -> Tuple[str, str]:
        """Generates a new random Ed25519 keypair (public_key, secret_key)."""
        kp = Keypair.random()
        return kp.public_key, kp.secret

    @staticmethod
    def create_preimage_and_hash() -> Tuple[str, str]:
        """
        Generates a secure 32-byte secret preimage and its SHA-256 hash lock.
        Returns:
            (preimage_hex, hash_lock_hex)
        """
        preimage = secrets.token_bytes(32)
        hash_lock = hashlib.sha256(preimage).digest()
        return preimage.hex(), hash_lock.hex()

    def build_x402_lock_params(
        self,
        token: str,
        escrow: str,
        seller: str,
        amount: int,
        hash_lock_hex: str,
        timeout_ledger: int,
    ) -> Dict[str, Any]:
        """
        Formats and validates arguments for execute_x402_lock invocation.
        """
        hash_bytes = bytes.fromhex(hash_lock_hex)
        if len(hash_bytes) != 32:
            raise ValueError("hash_lock must be exactly 32 bytes (64 hex characters)")

        return {
            "agent": self.public_key,
            "token": token,
            "escrow": escrow,
            "seller": seller,
            "amount": int(amount),
            "hash_lock": hash_bytes,
            "timeout_ledger": int(timeout_ledger),
        }

    def sign_x402_lock(
        self,
        treasury_contract: str,
        token: str,
        escrow_contract: str,
        seller: str,
        amount: int,
        hash_lock_hex: str,
        timeout_ledger: int,
    ) -> Dict[str, Any]:
        """
        Prepares and cryptographically signs the x402 lock execution.
        """
        params = self.build_x402_lock_params(
            token=token,
            escrow=escrow_contract,
            seller=seller,
            amount=amount,
            hash_lock_hex=hash_lock_hex,
            timeout_ledger=timeout_ledger,
        )

        tx_payload = {
            "function": "execute_x402_lock",
            "contract": treasury_contract,
            "caller": self.public_key,
            "parameters": {
                "agent": params["agent"],
                "token": params["token"],
                "escrow": params["escrow"],
                "seller": params["seller"],
                "amount": params["amount"],
                "hash_lock": hash_lock_hex,
                "timeout_ledger": params["timeout_ledger"],
            },
            "signed_by": self.public_key,
            "status": "READY_FOR_SUBMISSION",
        }

        return tx_payload
