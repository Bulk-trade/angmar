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

# Export the Program ID as an environment variable and run npm run local
PROGRAM_ID=$program_id npm run local

# Wait for the server to start
sleep 10

# Print instructions to run the test script
echo "Server started. Run './test.sh' in a separate terminal to execute the API requests."