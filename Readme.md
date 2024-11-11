# Angmar

<div align="center">
  <img height="120px" src="https://avatars.githubusercontent.com/u/166141687?s=400&u=6dadcbbe30e0b6586f60c185aad22cd5d0fe8939&v=4" />

  <h1 style="margin-top:20px;">Angmar</h1>

  <p>
    <a href="#"><img alt="Docs" src="https://img.shields.io/badge/docs-available-blueviolet" /></a>
    <a href="#"><img alt="Discord Chat" src="https://img.shields.io/discord/123456789?color=blueviolet" /></a>
    <a href="https://opensource.org/licenses/Apache-2.0"><img alt="License" src="https://img.shields.io/badge/license-Apache%202.0-blueviolet" /></a>
  </p>
</div>

## Overview

The Angmar project by Bulk Labs is an open-source initiative that provides access to cutting-edge Solana programs and a TypeScript SDK for interacting with decentralized finance protocols. This project integrates various components, including Rust and TypeScript codebases, to facilitate management and interaction with the Solana blockchain.

## Requirements

- **Solana CLI**: v1.16.3
- **Platform Tools**: v1.37
- **Rust Compiler**: rustc 1.68.0 or higher

You can verify your Rust compiler version with:
```bash
rustc --version
```

### Setup Instructions
**Setup Script**
To initialize the project environment, run:
```bash
./setup.sh
```

### Cleanup
To clean up the project directories and remove build artifacts, execute:
```bash
./cleanup.sh
```

### Local Validator Setup
Set up a local Solana validator and run tests using:
```bash
anchor test
```

### Local Deployment
To deploy the program locally, run:
```bash
./deploy.sh
```

### Test Script
To execute the test scripts, use:
```bash
./test.sh
```

### Project Structure
```bash
.
├── .anchor/                  # Anchor framework configurations and logs
├── api/                      # API server code (TypeScript)
├── deps/                     # Dependencies
├── idl/                      # Interface Definition Language files for smart contracts
├── scripts/                  # Various scripts
├── vault-program/            # Rust code for the vault program
├── setup.sh                  # Setup script
├── cleanup.sh                # Cleanup script
├── deploy.sh                 # Deployment script
├── test.sh                   # Test execution script
└── README.md                 # 
```

### Project documentation

**Building Locally**
Note: If you're using an Apple computer with an M1 chip, set the default Rust toolchain to stable-x86_64-apple-darwin:
```bash
rustup default stable-x86_64-apple-darwin
```

### Resources
**Documentation:** Coming Soon
**Discord Community:** Join Here
***License:** Licensed under the Apache-2.0 License.
License
This project is licensed under the Apache-2.0 License.