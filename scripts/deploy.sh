#!/bin/bash
set -e

echo "Deploying Agent Treasury..."
TREASURY_ID=$(soroban contract deploy --wasm target/wasm32-unknown-unknown/release/agent_treasury.wasm --source admin --network testnet)
echo "Treasury ID: $TREASURY_ID"

echo "Deploying x402 Escrow..."
ESCROW_ID=$(soroban contract deploy --wasm target/wasm32-unknown-unknown/release/x402_escrow.wasm --source admin --network testnet)
echo "Escrow ID: $ESCROW_ID"

echo "Generating TypeScript Bindings..."
soroban contract bindings typescript --network testnet --contract-id $TREASURY_ID --output-dir ../voxtrade-app/packages/treasury-sdk
soroban contract bindings typescript --network testnet --contract-id $ESCROW_ID --output-dir ../voxtrade-app/packages/escrow-sdk

