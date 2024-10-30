#!/bin/bash

#remove old test ledger
rm -rf test-ledger   

#start solana validator
solana-test-validator --clone-upgradeable-program dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH --clone-upgradeable-program FsJ3A3u2vn5cTVofAjvy6y5kwABJAqYWpe4975bi2epH --clone-upgradeable-program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s --url https://api.mainnet-beta.solana.com