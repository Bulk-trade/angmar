#!/bin/bash

# Remove the target directory
# rm -rf target

#Set localhost
#solana config set --url localhost

# Update deps if not updated
# https://solana.stackexchange.com/questions/8800/error-use-of-unstable-library-feature-build-hasher-simple-hash-one
# cargo update -p ahash@0.8.11 --precise 0.8.6

# Update for drift
# cargo update -p bumpalo@3.16.0 --precise 3.14.0
# cargo update -p anchor-lang@0.30.0 --precise 0.29.0

#rm '/Users/mac/Desktop/BULK/angmar/target/deploy/vault_program-keypair.json'

# Build the Solana program
cargo build-sbf --tools-version v1.41  

# Check if the build was successful
if [ $? -eq 0 ]; then
# Deploy the Solana program and capture the output
deploy_output=$(solana program deploy ./target/deploy/vault_program.so)

# Extract the Program ID from the output using awk
program_id=$(echo "$deploy_output" | awk '/Program Id:/ {print $3}')

# Generate IDL with shank cli
shank idl -p $program_id

# Print the Program ID
echo "Program ID: $program_id"

# Navigate to the api directory
cd api

# Print instructions to run the test script
echo "Starting Server. Run './test.sh' in a separate terminal to execute the API requests."

# Export the Program ID as an environment variable and run npm run local
echo "PROGRAM_ID=$program_id npm run local"
PROGRAM_ID=$program_id npm run local

else
    echo "Build failed. Deployment aborted."
    exit 1
fi


