#!/bin/bash

# Initialize the vault
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

# Initialize the drift vault with bulk
# echo "Initializing drift vault..."
# curl -X POST http://localhost:4001/init-drift-bulk \
#      -H "Content-Type: application/json" \
#      -d '{"name": "bulk_vault"}'
# echo ""

# # Deposit into the vault
echo "Depositing usdc into vault..."
curl -X POST http://localhost:4001/deposit-usdc \
     -H "Content-Type: application/json" \
     -d '{"vault_id": "bulk_vault", "user_pubkey": "sunit", "amount": 10000}'
echo ""

# # Withdraw from the vault
# echo "Withdrawing from vault..."
# curl -X POST http://localhost:4001/withdraw-usdc \
#      -H "Content-Type: application/json" \
#      -d '{"vault_id": "bulk_vault", "user_pubkey": "sunit", "amount": 9000}'
# echo ""

# echo "Updating Delegate"
# curl -X POST http://localhost:4001/update-delegate \
#      -H "Content-Type: application/json" \
#      -d '{"vault_id": "bulk_vault", "delegate": "CCS5GBTmg9VMsuwXRXPBDeYAiJgjme6FUE1vVammsEeC", "sub_account": 0}'
# echo ""