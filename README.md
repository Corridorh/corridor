# Soroban Travel-Rule Compliant Payment Router

A smart contract system for [Soroban](https://developers.stellar.org/docs/build/smart-contracts) (Stellar's smart contract platform) that lets regulated entities — exchanges, custodians, and other Virtual Asset Service Providers (VASPs) — enforce FATF Travel-Rule-style compliance metadata on token transfers, without ever putting personal data on-chain.

## The Problem

Stellar's [SEP-41 token interface](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0041.md) — the standard used by the Stellar Asset Contract and virtually every fungible token on Soroban — defines `transfer(from, to, amount)` with no metadata field. That's correct for the interface's purpose, but it means there's currently no on-chain mechanism for two VASPs to prove that required originator/beneficiary information (the "Travel Rule," per FATF Recommendation 16) was exchanged before a large transfer settles.

Today, VASPs on Stellar either handle this entirely off-chain with no on-chain enforcement, or don't handle it at all.

## The Approach

This project does **not** put personal data on the Stellar ledger. Instead it follows the pattern used by real-world travel-rule protocols (TRISA, TRP, Sygna): a **hash-commitment on-chain, encrypted payload exchange off-chain.**

```
┌────────────┐        1. Exchange encrypted           ┌────────────┐
│  VASP A    │ ─────── IVMS101-lite payload ─────────▶ │  VASP B    │
│ (sender)   │ ◀────── off-chain (HTTPS) ───────────── │ (receiver) │
└─────┬──────┘                                          └─────┬──────┘
      │ 2. hash(payload)                                      │
      ▼                                                        │
┌─────────────────────────────────────────────────────────────┴───┐
│                     Soroban Network (on-chain)                   │
│                                                                    │
│  ┌──────────────────┐        checks         ┌───────────────┐   │
│  │  router contract  │ ────────────────────▶ │ vasp-registry │   │
│  │  compliant_transfer│                       │   contract    │   │
│  └─────────┬──────────┘                       └───────────────┘   │
│            │ wraps                                                │
│            ▼                                                      │
│  ┌──────────────────┐                                             │
│  │  SEP-41 token     │  (e.g. Stellar Asset Contract, USDC, etc.) │
│  └──────────────────┘                                             │
└────────────────────────────────────────────────────────────────┘
```

1. Below a configurable threshold, transfers pass straight through to the underlying token — no overhead for everyday payments.
2. At or above the threshold, the sending VASP must supply a `metadata_hash` — a SHA-256 commitment to the compliance payload that was exchanged off-chain with the counterparty.
3. The router verifies both parties are registered, active VASPs (via the `vasp-registry` contract) and that the hash hasn't been replayed, before releasing funds.
4. Anyone can later verify off-chain that a given hash corresponds to a legitimate, exchanged payload — without the payload itself ever touching the public ledger.

## Repo Structure

| Path | What it is |
|---|---|
| `contracts/router/` | Soroban contract wrapping a SEP-41 token with threshold-gated compliance checks |
| `contracts/vasp-registry/` | Soroban contract maintaining the allowlist of registered VASPs |
| `contracts/common/` | Shared Rust types/errors used by both contracts |
| `offchain/metadata-exchange/` | Reference Node/TypeScript service showing how VASPs exchange encrypted payloads and compute hashes |
| `sdk/ts/` | TypeScript SDK for building and submitting `compliant_transfer` transactions |
| `docs/architecture.md` | Deeper technical walkthrough of the full flow |
| `tests/integration/` | End-to-end tests exercising the whole stack against a local/testnet deployment |

Full technical build spec (contract interfaces, storage layout, function signatures, test requirements) lives in [`BUILD-SPEC.md`](./BUILD-SPEC.md).

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable) with the `wasm32-unknown-unknown` target:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
- [Stellar CLI](https://developers.stellar.org/docs/tools/cli/install-cli) (formerly `soroban-cli`)
- [Node.js](https://nodejs.org/) 18+ (for the off-chain service and SDK)
- A funded Soroban Testnet account ([friendbot](https://developers.stellar.org/docs/tools/quickstart-with-labs/create-account))

## Getting Started

### 1. Clone and build the contracts

```bash
git clone https://github.com/code3ks/corridor.git
cd corridor
cargo build --target wasm32-unknown-unknown --release
```

### 2. Run contract tests

```bash
cargo test
```

### 3. Deploy to Testnet

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/vasp_registry.wasm \
  --source <your-account> \
  --network testnet

stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/router.wasm \
  --source <your-account> \
  --network testnet
```

Then initialize both contracts (admin, token address, registry address, threshold) — see `docs/architecture.md` for exact invoke commands.

### 4. Run the off-chain reference service

```bash
cd offchain/metadata-exchange
npm install
npm run dev
```

### 5. Use the SDK

```bash
cd sdk/ts
npm install
npm run build
```

```ts
import { hashPayload, buildCompliantTransferTx } from "soroban-travel-rule-sdk";

const hash = hashPayload({
  originator: { name: "Alice", vasp_id: "VASP-A", account_ref: "acct-1" },
  beneficiary: { name: "Bob", vasp_id: "VASP-B", account_ref: "acct-2" },
  amount: "5000",
  asset_code: "USDC",
  timestamp: new Date().toISOString(),
});

const tx = await buildCompliantTransferTx({
  routerContractId: "C...",
  from: "G...",
  to: "G...",
  amount: "5000",
  metadataHash: hash,
  networkPassphrase: "Test SDF Network ; September 2015",
  rpcUrl: "https://soroban-testnet.stellar.org",
});
```

## Security & Compliance Notes

- **No PII on-chain, ever.** Only SHA-256 hashes and VASP routing metadata (endpoint URLs) are stored on the Stellar ledger.
- This is a **reference implementation**, not an audited, production-ready compliance system. Encryption and key management in the off-chain service are stubbed and marked with `TODO` comments — real deployments need proper KMS/HSM integration and mutual TLS between VASPs.
- The `vasp-registry` contract uses a single-admin model for v1. Production deployments should consider a multisig or governance-controlled admin.
- This project implements an IVMS101-*lite* payload for MVP purposes, not the full IVMS101 schema.

See [`BUILD-SPEC.md`](./BUILD-SPEC.md) for the complete list of v1 scope and explicit non-goals.

## Contributing

Contributions are welcome, including through the [Drips Wave](https://www.drips.network/wave/stellar) Stellar program. Check the Issues tab for scoped tasks tagged with point values. Good first issues are labeled `good-first-issue`.

To contribute:
1. Fork the repo and create a feature branch
2. Make your changes with tests
3. Run `cargo test` and `npm test` (in `sdk/ts` and `offchain/metadata-exchange`) before opening a PR
4. Open a PR describing what changed and why

## License

[MIT](./LICENSE) — see LICENSE file for details.

## References

- [SEP-41: Soroban Token Interface](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0041.md)
- [Soroban Docs](https://developers.stellar.org/docs/build/smart-contracts)
- [FATF Travel Rule (Recommendation 16)](https://www.fatf-gafi.org/)
- [TRISA Protocol](https://trisa.io/) (reference for the on-chain/off-chain pattern used here)
