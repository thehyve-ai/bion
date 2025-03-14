# BION

## ⚠️ ALPHA SOFTWARE WARNING ⚠️

> **DISCLAIMER**: Bion is currently in ALPHA stage. This software is experimental and under active development.
>
> - **USE AT YOUR OWN RISK**: This software may contain bugs, errors, or security vulnerabilities.
> - **NO WARRANTY**: The software is provided "as is", without warranty of any kind, express or implied.
> - **POTENTIAL FINANCIAL LOSS**: Interacting with blockchain networks involves risk. You could lose funds.
> - **AUDIT STATUS**: This software has not undergone a comprehensive security audit.
> - **BREAKING CHANGES**: APIs and functionality may change significantly between versions.
>
> By using Bion, you acknowledge and accept these risks. We strongly recommend testing with small amounts on testnets before using on mainnet with significant value.

[gha-badge]: https://img.shields.io/github/actions/workflow/status/thehyve-ai/bion/test.yml?branch=master
[gha-url]: https://github.com/thehyve-ai/bion/actions
[tg-badge]: https://img.shields.io/endpoint?color=neon&logo=telegram&label=chat&style=flat-square&url=https%3A%2F%2Ftg.sumanjay.workers.dev%2Fbion_rs
[tg-url]: https://t.me/bion_rs

Bion is a CLI tool for managing entities on the Symbiotic Protocol. It uses Cast underneath to interact with your wallet and the chain, making it blazing fast, battle-tested, and secure. It supports multi-sig through [Safe](https://safe.global/) and all wallets that are supported by [Cast](https://github.com/foundry-rs/cast).

Here are a few examples of what you can do with Bion:

### Get information, including balances, of vaults

```bash
bion list-vaults --rpc-url https://eth.merkle.io
```

### Change the max-network-limit for a vault

```bash
bion network hyve set-max-network-limit <vault-address> <subnetwork> 12.56 --rpc-url https://eth.merkle.io
```

---

Run `bion --help` to explore the full list of available subcommands and their usage.

For help on how to use this CLI for your network or vault, please refer to the [usage](#usage) section below.

## Installation

Install `bion`:

```bash
curl -L https://bion.hyve.xyz | bash
```

Done! Now you can use `bion` to interact with your networks and vaults.

## Usage

Bion uses a command structure that allows you to interact with networks and vaults through aliases.

### Setting Up Aliases

Before using network, vault, or operator commands, you need to set up aliases for easier interaction:

#### Add an alias

`bion add-alias <alias> <network-manager-address | vault-admin-address | operator-address> --rpc-url <rpc-url>`

```bash
bion add-alias hyve 0xE3a148b25Cca54ECCBD3A4aB01e235D154f03eFa --rpc-url https://eth.merkle.io
```

#### Remove an alias

`bion remove-alias <alias> --rpc-url <rpc-url>`

```bash
bion remove-alias hyve --rpc-url https://eth.merkle.io
```

### Using aliases

Once you have set up your aliases, you can use them to interact with networks and vaults:

#### Example set a max network limit for a network and vault

`bion network hyve set-max-network-limit 0xc10A7f0AC6E3944F4860eE97a937C51572e3a1Da 0 1000.00 --rpc-url https://eth.merkle.io`

## Roadmap

🔨 = In Progress

🛠 = Feature complete. Additional testing required.

✅ = Feature complete

| Feature                                                   | Status |
| --------------------------------------------------------- | :----: |
| **Symbiotic**                                             |        |
| Deploy a vault                                            |   🔨   |
| Slash a vault from the CLI                                |   🔨   |
| Retrieve slashing events                                  |   🔨   |
| Deposit funds into a vault                                |   🔨   |
| Withdraw funds from a vault                               |   🔨   |
| Get and list information about networks                   |   🔨   |
| Rewards-specific commands                                 |   🔨   |
| **Network Specific**                                      |        |
| Support custom onboarding commands for arbitrary networks |   🔨   |
| **Security**                                              |        |
| Comprehensive e2e tests                                   |   🔨   |

## Acknowledgements

This project incorporates code from the following open source projects:

- **Foundry** - Significant portions of our code are derived from the Cast crate from the [Foundry](https://github.com/foundry-rs/foundry) project by Georgios Konstantopoulos, used under the MIT and Apache 2.0 licenses.
- **Lighthouse** - Components related to validator key management are adapted from [Lighthouse](https://github.com/sigp/lighthouse), used under the Apache 2.0 license.
- **Reth** - Our CLI runner is inspired by [Reth](https://github.com/paradigmxyz/reth), used under the MIT/Apache 2.0 licenses.

See the LICENSES directory and NOTICE file for detailed attribution and license information.
