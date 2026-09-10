"""
VoxTrade AI Voice Agent - Real-time Negotiation & Quota Engine
Author: ogundeleoluwaferanmi35
"""

import time
from enum import Enum
from typing import Dict, Any, Optional
from x402_signer import VoiceAgentSigner

class NegotiationState(str, Enum):
    DISCONNECTED = "DISCONNECTED"
    CONNECTING = "CONNECTING"
    NEGOTIATING = "NEGOTIATING"
    LOCKING_ESCROW = "LOCKING_ESCROW"
    STREAMING = "STREAMING"
    COMPLETED = "COMPLETED"
    FAILED = "FAILED"

class VoiceNegotiationEngine:
    """
    State machine that manages real-time voice streaming negotiations,
    guaranteeing that spending never exceeds the daily treasury limit.
    """

    def __init__(
        self,
        signer: VoiceAgentSigner,
        treasury_contract_id: str,
        daily_limit_stroops: int = 100_000_000, # 10 USDC
    ):
        self.signer = signer
        self.treasury_contract_id = treasury_contract_id
        self.daily_limit_stroops = daily_limit_stroops
        self.spent_today_stroops = 0
        self.state = NegotiationState.DISCONNECTED
        self.current_session: Optional[Dict[str, Any]] = None

    def evaluate_rate(self, proposed_rate_stroops: int) -> bool:
        """
        Determines if proposed rate is acceptable based on quota and heuristics.
        """
        if self.spent_today_stroops + proposed_rate_stroops > self.daily_limit_stroops:
            return False
        # Maximum rate per 2-second chunk is 0.10 USDC (1,000,000 stroops)
        return proposed_rate_stroops <= 1_000_000

    def start_negotiation(
        self,
        supplier_id: str,
        requested_service: str = "voice_translation",
    ) -> Dict[str, Any]:
        """
        Initiates a voice negotiation session with a supplier agent.
        """
        self.state = NegotiationState.NEGOTIATING
        self.current_session = {
            "session_id": f"sess_{int(time.time())}",
            "supplier": supplier_id,
            "service": requested_service,
            "created_at": time.time(),
        }
        return {
            "status": "NEGOTIATING",
            "agent_key": self.signer.public_key,
            "daily_budget_remaining": self.daily_limit_stroops - self.spent_today_stroops,
        }

    def commit_x402_chunk(
        self,
        token_address: str,
        escrow_contract: str,
        amount_stroops: int,
        hash_lock: str,
        timeout_ledger: int,
    ) -> Dict[str, Any]:
        """
        Signs the on-chain lock transaction for a streaming audio chunk.
        """
        if not self.evaluate_rate(amount_stroops):
            self.state = NegotiationState.FAILED
            raise ValueError(f"Amount {amount_stroops} exceeds remaining daily treasury quota")

        self.state = NegotiationState.LOCKING_ESCROW
        tx_payload = self.signer.sign_x402_lock(
            treasury_contract=self.treasury_contract_id,
            token=token_address,
            escrow_contract=escrow_contract,
            seller=self.current_session["supplier"],
            amount=amount_stroops,
            hash_lock_hex=hash_lock,
            timeout_ledger=timeout_ledger,
        )

        self.spent_today_stroops += amount_stroops
        self.state = NegotiationState.STREAMING

        return {
            "state": self.state.value,
            "tx_payload": tx_payload,
            "spent_today": self.spent_today_stroops,
            "remaining_quota": self.daily_limit_stroops - self.spent_today_stroops,
        }
