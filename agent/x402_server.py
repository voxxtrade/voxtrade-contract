"""
VoxTrade x402 HTTP 402 Micropayment Server & Voice Streamer
Author: ogundeleoluwaferanmi35
"""

import hashlib
import secrets
from typing import Dict, Any, Optional
from fastapi import FastAPI, Header, HTTPException, Response, status
from pydantic import BaseModel

app = FastAPI(
    title="VoxTrade x402 Micropayment Server",
    description="Implements HTTP 402 Payment Required for AI Voice-to-Voice Commerce on Stellar",
    version="1.0.0",
)

# Simulated in-memory session database
SESSIONS: Dict[str, Dict[str, Any]] = {}
ESCROW_CONTRACT_ID = "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"
USDC_TOKEN_ID = "CUSDC...MOCK_TOKEN"

class NegotiateRequest(BaseModel):
    buyer_agent: str
    service_type: str = "voice_synthesis_48khz"
    max_rate_stroops: int = 1000000

class NegotiateResponse(BaseModel):
    session_id: str
    price_per_chunk: int
    chunk_duration_sec: int
    hash_lock: str
    escrow_contract: str
    token: str
    timeout_ledgers: int

@app.get("/health")
def health_check():
    return {"status": "ONLINE", "protocol": "x402/v1.0", "network": "stellar-testnet"}

@app.post("/api/negotiate", response_model=NegotiateResponse)
def negotiate_terms(req: NegotiateRequest):
    """
    AI Agent calls this endpoint to negotiate streaming price terms.
    Generates a secret preimage and hash lock for the session.
    """
    session_id = secrets.token_hex(8)
    preimage = secrets.token_bytes(32)
    hash_lock = hashlib.sha256(preimage).hexdigest()

    price_per_chunk = min(req.max_rate_stroops, 500000) # 0.05 USDC per 2-second chunk

    SESSIONS[session_id] = {
        "buyer_agent": req.buyer_agent,
        "service_type": req.service_type,
        "price_per_chunk": price_per_chunk,
        "preimage": preimage.hex(),
        "hash_lock": hash_lock,
        "escrow_id": None,
        "paid": False,
    }

    return NegotiateResponse(
        session_id=session_id,
        price_per_chunk=price_per_chunk,
        chunk_duration_sec=2,
        hash_lock=hash_lock,
        escrow_contract=ESCROW_CONTRACT_ID,
        token=USDC_TOKEN_ID,
        timeout_ledgers=120,
    )

@app.get("/api/audio/stream/{session_id}/{chunk_id}")
def stream_audio_chunk(
    session_id: str,
    chunk_id: int,
    authorization: Optional[str] = Header(None),
):
    """
    Protected streaming endpoint.
    Returns HTTP 402 if unpaid, or returns binary audio + X-Preimage if paid.
    """
    session = SESSIONS.get(session_id)
    if not session:
        raise HTTPException(status_code=404, detail="Session expired or not found")

    # If unauthenticated or no x402 proof provided, return 402 Payment Required
    if not authorization or not authorization.startswith("x402 "):
        headers = {
            "WWW-Authenticate": (
                f'x402 contract="{ESCROW_CONTRACT_ID}", '
                f'token="{USDC_TOKEN_ID}", '
                f'amount="{session["price_per_chunk"]}", '
                f'hash_lock="{session["hash_lock"]}", '
                f'timeout_ledgers="120"'
            )
        }
        return Response(
            status_code=status.HTTP_402_PAYMENT_REQUIRED,
            headers=headers,
            content=f'{{"error": "Payment Required", "protocol": "x402", "amount": {session["price_per_chunk"]}, "hash_lock": "{session["hash_lock"]}"}}',
            media_type="application/json",
        )

    # Client provided payment proof (e.g. "x402 escrow_id=42, tx_hash=0x...")
    session["paid"] = True

    # 100 bytes of dummy Opus audio frame
    dummy_audio_bytes = b"\x4f\x70\x75\x73\x00" * 20

    response_headers = {
        "X-Preimage": session["preimage"],
        "X-Session-Id": session_id,
        "X-Chunk-Id": str(chunk_id),
        "Content-Type": "audio/opus",
    }

    return Response(content=dummy_audio_bytes, headers=response_headers, media_type="audio/opus")
