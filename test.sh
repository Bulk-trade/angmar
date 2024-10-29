#!/bin/bash

Initialize the vault
echo "Initializing vault..."
curl -X POST http://localhost:4001/initVault \
     -H "Content-Type: application/json" \
     -d '{"vault_id": "bulk_vault"}'
echo ""


# Initialize the drift vault
echo "Initializing drift vault..."
curl -X POST http://localhost:4001/initDrift \
     -H "Content-Type: application/json" \
     -d '{"vault_id": "bulk_vault"}'
echo ""

# Deposit into the vault
# echo "Depositing into vault..."
# curl -X POST http://localhost:4001/deposit \
#      -H "Content-Type: application/json" \
#      -d '{"vault_id": "bulk_vault", "user_pubkey": "sunit", "amount": 1.0}'
# echo ""

# # Withdraw from the vault
# echo "Withdrawing from vault..."
# curl -X POST http://localhost:4001/withdraw \
#      -H "Content-Type: application/json" \
#      -d '{"vault_id": "bulk_vault", "user_pubkey": "sunit", "amount": 1.0}'
# echo ""