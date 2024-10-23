#!/bin/bash

# Remove the target directory
rm -rf target

# Build the Solana program
cargo build-sbf

# Deploy the Solana program and capture the output
deploy_output=$(solana program deploy ./target/deploy/vault_program.so)

# Extract the Program ID from the output using awk
program_id=$(echo "$deploy_output" | awk '/Program Id:/ {print $3}')

# Print the Program ID
echo "Program ID: $program_id"

# Navigate to the api directory
cd api

# Print instructions to run the test script
echo "Starting Server. Run './test.sh' in a separate terminal to execute the API requests."

# Export the Program ID as an environment variable and run npm run local
echo "PROGRAM_ID=$program_id npm run local"
PROGRAM_ID=$program_id npm run local


