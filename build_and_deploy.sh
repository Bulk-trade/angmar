#!/bin/bash

# Remove the target directory
rm -rf target

# Build the Solana program
cargo build-sbf

# Deploy the Solana program
solana program deploy ./target/deploy/vault_program.so