# Deployment Guide

This guide deploys Corridor to Stellar Testnet. The same steps apply to
Mainnet, but do not deploy to Mainnet until the limitations in
[`SECURITY.md`](../SECURITY.md) are resolved and the contracts have been
audited.

## 1. Prerequisites

- Rust stable with the `wasm32-unknown-unknown` target:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
- The [Stellar CLI](https://developers.stellar.org/docs/tools/cli/install-cli)
  (`stellar`, formerly `soroban-cli`) v21 or newer.
- A funded Testnet identity:
  ```bash
  stellar keys generate --global admin --network testnet
  stellar keys fund admin --network testnet
  ```

## 2. Build the contracts

From the repository root:

```bash
cargo build --workspace --target wasm32-unknown-unknown --release
```

Artifacts:

```
target/wasm32-unknown-unknown/release/router.wasm
target/wasm32-unknown-unknown/release/vasp_registry.wasm
```

Alternatively, `stellar contract build` produces the same artifacts under
`target/wasm32-unknown-unknown/release/`.

## 3. Deploy the registry

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/vasp_registry.wasm \
  --source admin \
  --network testnet \
  --alias vasp-registry
```

Initialize it. Initialization requires the admin's authorization:

```bash
stellar contract invoke \
  --id vasp-registry \
  --source admin \
  --network testnet \
  -- initialize \
  --admin "$(stellar keys address admin)"
```

## 4. Deploy the router

You need the address of the SEP-41 token to wrap. For a quick Testnet run you
can deploy a Stellar Asset Contract, or use an existing token.

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/router.wasm \
  --source admin \
  --network testnet \
  --alias corridor-router

stellar contract invoke \
  --id corridor-router \
  --source admin \
  --network testnet \
  -- initialize \
  --admin "$(stellar keys address admin)" \
  --token "$TOKEN_ADDRESS" \
  --registry "$(stellar contract alias show vasp-registry)" \
  --threshold 1000000000
```

`threshold` is expressed in the token's smallest unit. With 7-decimal USDC,
`1000000000` is 100 USDC.

## 5. Register VASPs

```bash
stellar contract invoke \
  --id vasp-registry \
  --source admin \
  --network testnet \
  -- add_vasp \
  --caller "$(stellar keys address admin)" \
  --vasp "$VASP_A_ADDRESS" \
  --endpoint_url "https://vasp-a.example.com/travelrule"

stellar contract invoke \
  --id vasp-registry \
  --source admin \
  --network testnet \
  -- add_vasp \
  --caller "$(stellar keys address admin)" \
  --vasp "$VASP_B_ADDRESS" \
  --endpoint_url "https://vasp-b.example.com/travelrule"
```

## 6. Verify the deployment

```bash
stellar contract invoke --id vasp-registry --network testnet -- get_admin
stellar contract invoke --id vasp-registry --network testnet -- is_registered --vasp "$VASP_A_ADDRESS"
stellar contract invoke --id corridor-router --network testnet -- get_threshold
stellar contract invoke --id corridor-router --network testnet -- get_registry
```

Then build a transfer with the SDK:

```ts
import { hashPayload, buildCompliantTransferTx } from "soroban-travel-rule-sdk";

const metadataHash = hashPayload({
  originator: { name: "Alice", vasp_id: "VASP-A", account_ref: "acct-1" },
  beneficiary: { name: "Bob", vasp_id: "VASP-B", account_ref: "acct-2" },
  amount: "5000",
  asset_code: "USDC",
  timestamp: new Date().toISOString(),
});

const xdr = await buildCompliantTransferTx({
  routerContractId: "<ROUTER_CONTRACT_ID>",
  from: "<VASP_A_ADDRESS>",
  to: "<VASP_B_ADDRESS>",
  amount: "5000",
  metadataHash,
  networkPassphrase: "Test SDF Network ; September 2015",
  rpcUrl: "https://soroban-testnet.stellar.org",
});
```

Sign the returned XDR and submit it with the Stellar SDK or CLI.

## 7. Deployment checklist

- [ ] `cargo test --workspace` passes.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [ ] Both WASM artifacts are under the 64 KiB budget.
- [ ] Registry and router are initialized with the intended admin.
- [ ] `get_admin`, `get_token`, `get_registry`, and `get_threshold` return the
      expected values.
- [ ] At least two VASPs are registered and `is_registered` returns `true`.
- [ ] A below-threshold transfer succeeds without a hash.
- [ ] An at-threshold transfer without a hash fails with `MissingMetadata`.
- [ ] An at-threshold transfer to an unregistered address fails with
      `UnregisteredCounterparty`.
- [ ] Replaying a hash fails with `HashAlreadyUsed`.
- [ ] The deployment record (contract IDs, commit SHA, network, admin) is
      committed under [`deployments/`](../deployments/README.md).

## 8. Upgrade policy

The contracts are immutable and expose no upgrade entry point. To change the
policy:

1. Deploy a new router with the new code.
2. Register the same VASPs in a new registry (or point the new router at the
   existing registry).
3. Migrate traffic and drain the old router.
4. Publish the new deployment record and update the SDK defaults.

Never attempt to mutate a deployed contract's storage layout in place.
