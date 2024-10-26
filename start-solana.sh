#!/bin/bash

#remove old test ledger
rm -rf test-ledger   

#start solana validator
solana-test-validator --clone-upgradeable-program dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH --url https://api.mainnet-beta.solana.com