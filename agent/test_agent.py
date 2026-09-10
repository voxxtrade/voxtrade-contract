"""
Unit Test Suite for VoxTrade Autonomous Voice Agent Runtime
Tests: Keypair generation, preimage derivation, HTLC parameter construction,
negotiation state machine transitions, and daily treasury quota limits.
Author: ogundeleoluwaferanmi35
"""

import unittest
import hashlib
import sys
import os

# Add agent directory to sys.path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from x402_signer import VoiceAgentSigner
from negotiation_engine import VoiceNegotiationEngine, NegotiationState

class TestVoiceAgentSigner(unittest.TestCase):
    def setUp(self):
        self.signer = VoiceAgentSigner()
        self.treasury_id = "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"
        self.token_id = "CCUSDC4SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"
        self.escrow_id = "CESCROW3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"
        self.seller_id = "GBZXN7PIRZGNMHGA72ST2EQTVGQDXF45NWZ46CXCW462KC22W6KZNVLB"

    def test_keypair_generation(self):
        pub, sec = VoiceAgentSigner.generate_keypair()
        self.assertTrue(pub.startswith("G"))
        self.assertTrue(sec.startswith("S"))
        self.assertEqual(len(pub), 56)
        self.assertEqual(len(sec), 56)

    def test_preimage_and_hash(self):
        preimage_hex, hash_hex = VoiceAgentSigner.create_preimage_and_hash()
        self.assertEqual(len(preimage_hex), 64)
        self.assertEqual(len(hash_hex), 64)

        # Verify hash relationship
        expected_hash = hashlib.sha256(bytes.fromhex(preimage_hex)).hexdigest()
        self.assertEqual(hash_hex, expected_hash)

    def test_build_x402_lock_params_valid(self):
        _, hash_hex = VoiceAgentSigner.create_preimage_and_hash()
        params = self.signer.build_x402_lock_params(
            token=self.token_id,
            escrow=self.escrow_id,
            seller=self.seller_id,
            amount=500_000,
            hash_lock_hex=hash_hex,
            timeout_ledger=1000,
        )
        self.assertEqual(params["amount"], 500_000)
        self.assertEqual(params["seller"], self.seller_id)
        self.assertEqual(len(params["hash_lock"]), 32)
        self.assertEqual(params["timeout_ledger"], 1000)

    def test_build_x402_lock_params_invalid_hash(self):
        with self.assertRaises(ValueError):
            self.signer.build_x402_lock_params(
                token=self.token_id,
                escrow=self.escrow_id,
                seller=self.seller_id,
                amount=500_000,
                hash_lock_hex="1234abcd", # Not 32 bytes
                timeout_ledger=1000,
            )

    def test_sign_x402_lock(self):
        _, hash_hex = VoiceAgentSigner.create_preimage_and_hash()
        tx_payload = self.signer.sign_x402_lock(
            treasury_contract=self.treasury_id,
            token=self.token_id,
            escrow_contract=self.escrow_id,
            seller=self.seller_id,
            amount=250_000,
            hash_lock_hex=hash_hex,
            timeout_ledger=1200,
        )
        self.assertEqual(tx_payload["function"], "execute_x402_lock")
        self.assertEqual(tx_payload["contract"], self.treasury_id)
        self.assertEqual(tx_payload["status"], "READY_FOR_SUBMISSION")
        self.assertEqual(tx_payload["parameters"]["amount"], 250_000)
        self.assertEqual(tx_payload["parameters"]["hash_lock"], hash_hex)


class TestVoiceNegotiationEngine(unittest.TestCase):
    def setUp(self):
        self.signer = VoiceAgentSigner()
        self.treasury_id = "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"
        self.token_id = "CCUSDC4SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"
        self.escrow_id = "CESCROW3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"
        self.seller_id = "GBZXN7PIRZGNMHGA72ST2EQTVGQDXF45NWZ46CXCW462KC22W6KZNVLB"
        self.engine = VoiceNegotiationEngine(
            signer=self.signer,
            treasury_contract_id=self.treasury_id,
            daily_limit_stroops=10_000_000, # 1 USDC limit for test
        )

    def test_initial_state(self):
        self.assertEqual(self.engine.state, NegotiationState.DISCONNECTED)
        self.assertEqual(self.engine.spent_today_stroops, 0)

    def test_start_negotiation(self):
        res = self.engine.start_negotiation(supplier_id=self.seller_id)
        self.assertEqual(self.engine.state, NegotiationState.NEGOTIATING)
        self.assertEqual(res["status"], "NEGOTIATING")
        self.assertEqual(res["daily_budget_remaining"], 10_000_000)

    def test_rate_evaluation(self):
        # 100,000 stroops <= 1,000,000 cap and within 10,000,000 budget
        self.assertTrue(self.engine.evaluate_rate(100_000))
        # Exceeds max rate per chunk (1,000,000)
        self.assertFalse(self.engine.evaluate_rate(2_000_000))

    def test_commit_chunk_success(self):
        self.engine.start_negotiation(supplier_id=self.seller_id)
        _, hash_hex = VoiceAgentSigner.create_preimage_and_hash()

        res = self.engine.commit_x402_chunk(
            token_address=self.token_id,
            escrow_contract=self.escrow_id,
            amount_stroops=500_000,
            hash_lock=hash_hex,
            timeout_ledger=2000,
        )

        self.assertEqual(self.engine.state, NegotiationState.STREAMING)
        self.assertEqual(res["state"], "STREAMING")
        self.assertEqual(res["spent_today"], 500_000)
        self.assertEqual(res["remaining_quota"], 9_500_000)

    def test_quota_exceeded_rejection(self):
        self.engine.start_negotiation(supplier_id=self.seller_id)
        _, hash_hex = VoiceAgentSigner.create_preimage_and_hash()

        # Set budget to 200,000 stroops
        self.engine.daily_limit_stroops = 200_000

        with self.assertRaises(ValueError) as ctx:
            self.engine.commit_x402_chunk(
                token_address=self.token_id,
                escrow_contract=self.escrow_id,
                amount_stroops=500_000, # Exceeds 200,000
                hash_lock=hash_hex,
                timeout_ledger=2000,
            )
        self.assertIn("exceeds remaining daily treasury quota", str(ctx.exception))
        self.assertEqual(self.engine.state, NegotiationState.FAILED)

    def test_multiple_chunks_accumulation(self):
        self.engine.start_negotiation(supplier_id=self.seller_id)

        for i in range(5):
            _, hash_hex = VoiceAgentSigner.create_preimage_and_hash()
            res = self.engine.commit_x402_chunk(
                token_address=self.token_id,
                escrow_contract=self.escrow_id,
                amount_stroops=200_000,
                hash_lock=hash_hex,
                timeout_ledger=2000 + i,
            )
            self.assertEqual(res["spent_today"], 200_000 * (i + 1))

        self.assertEqual(self.engine.spent_today_stroops, 1_000_000)
        self.assertEqual(self.engine.daily_limit_stroops - self.engine.spent_today_stroops, 9_000_000)


if __name__ == "__main__":
    unittest.main()
