# Integration tests

Corridor's end-to-end contract behaviour is exercised with the real Soroban
`testutils` host, not mocks. The cross-contract suite lives in
[`contracts/router/src/test.rs`](../../contracts/router/src/test.rs) because it
needs both the router and the registry clients.

## What is covered

| Scenario | Location |
|---|---|
| Registry initialization and admin transfer | `contracts/vasp-registry/src/test.rs` |
| VASP add / remove / reactivate | `contracts/vasp-registry/src/test.rs` |
| Below-threshold pass-through | `contracts/router/src/test.rs` |
| At/above-threshold metadata enforcement | `contracts/router/src/test.rs` |
| Hash replay protection | `contracts/router/src/test.rs` |
| Registered / unregistered / removed counterparties | `contracts/router/src/test.rs` |
| Non-positive amounts and negative thresholds | `contracts/router/src/test.rs` |
| Canonical hashing and payload round-trip | `sdk/ts/src/crypto.test.ts`, `offchain/metadata-exchange/src/crypto.test.ts` |

Run everything:

```bash
cargo test --workspace
(cd sdk/ts && npm test)
(cd offchain/metadata-exchange && npm test)
```

## Testnet smoke test

Contract tests cannot catch deployment or configuration mistakes. After a
Testnet deployment, run the checklist in
[`docs/DEPLOYMENT.md`](../../docs/DEPLOYMENT.md#7-deployment-checklist).

A scripted version can be built on top of the Stellar CLI; the manual checklist
is authoritative until then.

## Adding an integration test

1. Prefer the real registry and a Stellar Asset Contract over hand-written
   mocks.
2. Assert both the returned error *and* the unchanged balances, so a partial
   state change is caught.
3. Use deterministic 32-byte hashes (`BytesN::from_array`) rather than random
   values, so failures are reproducible.
