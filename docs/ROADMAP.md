# Roadmap and scoped work

This file turns the gaps identified in [`architecture.md`](./architecture.md)
into concrete, independently reviewable tasks. Each task is written to be
copy-pasteable into a GitHub issue and is sized for the
[Drips Stellar Wave](https://docs.drips.network/wave/) program.

## How to use this list

1. Create a GitHub issue from the relevant section (the
   `good first issue` template lives in
   [`.github/ISSUE_TEMPLATE/good_first_issue.yml`](../.github/ISSUE_TEMPLATE/good_first_issue.yml)).
2. Add the `Stellar Wave` label, or add the issue to the Wave from the Drips
   dashboard and set the complexity.
3. Keep the acceptance criteria in the issue. A PR is mergeable only when they
   are all met.

Complexity maps to Drips Wave points: **Trivial** = 100, **Medium** = 150,
**High** = 200.

---

## P0 — correctness and security

### 1. Add signed delivery receipts to the off-chain protocol
**Complexity:** High · **Area:** off-chain, contracts

**Why:** The router only proves that a hash was supplied. It cannot prove the
counterparty received the payload.

**Scope:** Define a receipt (`exchange_id`, `hash`, signature over both), add a
`/v1/travel-rule/receipt` endpoint, and document how a sender verifies it before
submitting the transaction.

**Acceptance criteria:**
- [ ] Receipt schema documented in `BUILD-SPEC.md`.
- [ ] Receipts are signed with the VASP key and verified on receipt.
- [ ] Tests cover valid, tampered, and expired receipts.
- [ ] The deployment guide explains where the check fits.

### 2. Replace the encryption stub with KMS/HSM-backed ECIES
**Complexity:** High · **Area:** off-chain

**Why:** The reference service bundles the AES key with the ciphertext; it is
demonstration-only.

**Scope:** Encrypt to the recipient's public key (X25519 + HKDF + AES-256-GCM),
load keys from a KMS/HSM, and add key rotation.

**Acceptance criteria:**
- [ ] No symmetric key material is transmitted with the payload.
- [ ] Keys are loaded from an external provider, never hard-coded.
- [ ] Round-trip, wrong-key, and tamper tests pass.
- [ ] `SECURITY.md` no longer lists the stub as a limitation.

### 3. Move administration to a multisig / governance account
**Complexity:** High · **Area:** contracts, operations

**Why:** A single admin key can add or remove VASPs and change the threshold.

**Scope:** Document and test the use of a Stellar multisig account as the admin,
or add a small governance contract with an M-of-N approval flow.

**Acceptance criteria:**
- [ ] The admin can be a multisig account; an end-to-end test demonstrates it.
- [ ] Key-rotation runbook added to `docs/DEPLOYMENT.md`.
- [ ] No single key can silently change the allowlist.

### 4. Add an indexer/archiver for commitments and VASP history
**Complexity:** High · **Area:** off-chain

**Why:** Persistent entries can expire, and the ledger is not a convenient audit
store.

**Scope:** Subscribe to contract events (`cmp_xfer`, `vasp_add`, `vasp_rem`,
`admin_xfr`) and persist them in a queryable store with retention beyond the TTL
window.

**Acceptance criteria:**
- [ ] Events are ingested idempotently.
- [ ] Reorg/duplicate handling is documented.
- [ ] A query API exposes transfer-by-hash and VASP history.
- [ ] Tests cover reconnects and duplicate events.

### 5. Add a TTL heartbeat / monitoring job
**Complexity:** Medium · **Area:** operations

**Why:** A rarely-read registry entry can expire and silently de-list a VASP.

**Scope:** A small script or cron job that calls `get_admin`, `get_threshold`,
and `is_registered` for known VASPs on a schedule, and alerts on failures.

**Acceptance criteria:**
- [ ] Script is checked in under `scripts/`.
- [ ] It is safe to run repeatedly.
- [ ] The README documents how to schedule it.

---

## P1 — robustness

### 6. Property-based tests for payload canonicalization
**Complexity:** Medium · **Area:** SDK, off-chain

**Why:** A canonicalization mismatch between two VASPs produces a different
hash and a failed transfer.

**Scope:** Generate random nested payloads and assert that both
implementations produce identical digests for every key ordering.

**Acceptance criteria:**
- [ ] A shared test vector file is checked in.
- [ ] The TypeScript SDK and the service both consume it.
- [ ] At least 1,000 generated cases run in CI.

### 7. Fuzz the threshold and replay state machine
**Complexity:** Medium · **Area:** contracts

**Why:** Boundary conditions around `amount == threshold`, zero amounts, and
repeated hashes are security-sensitive.

**Scope:** Add a `cargo-fuzz` or proptest harness that drives
`compliant_transfer` with random amounts, thresholds, and hashes.

**Acceptance criteria:**
- [ ] Invariants are asserted (a hash is never consumed twice; balances only
      move on `Ok`).
- [ ] The harness runs in CI with a fixed seed budget.
- [ ] Any crash is minimised and turned into a regression test.

### 8. Validate and normalize VASP endpoint URLs
**Complexity:** Medium · **Area:** contracts

**Why:** Arbitrary strings are stored and later consumed by off-chain clients.

**Scope:** Enforce a bounded length, `https://` scheme, and a host; reject empty
or control characters. Decide whether normalization happens on-chain or in the
SDK.

**Acceptance criteria:**
- [ ] Invalid URLs are rejected with a documented error.
- [ ] Storage growth is bounded.
- [ ] Tests cover empty, over-long, and non-HTTPS values.

### 9. Add a sign-and-submit helper to the SDK
**Complexity:** Medium · **Area:** SDK

**Why:** `buildCompliantTransferTx` returns XDR but users must sign and submit it
themselves.

**Scope:** Add `signAndSubmitCompliantTransfer` with clear error mapping and
idempotency guidance.

**Acceptance criteria:**
- [ ] Helper returns the transaction hash and result.
- [ ] Simulation and submission failures are distinguishable.
- [ ] Unit tests use a mocked RPC server.

### 10. Map Soroban contract errors to typed SDK errors
**Complexity:** Medium · **Area:** SDK

**Why:** Callers currently receive raw simulation output.

**Scope:** Decode `RouterError` and `RegistryError` from failed simulations into
named TypeScript errors.

**Acceptance criteria:**
- [ ] Every error code has a named export and a doc comment.
- [ ] Unknown codes degrade gracefully.
- [ ] Tests cover each code.

### 11. Persist off-chain payloads in a real database
**Complexity:** Medium · **Area:** off-chain

**Why:** The reference service stores payloads in memory and loses them on
restart.

**Scope:** Add a storage interface with an encrypted database implementation
(SQLite or Postgres), migrations, and retention rules.

**Acceptance criteria:**
- [ ] Payloads are encrypted at rest.
- [ ] Retention/deletion is configurable.
- [ ] Integration tests use an ephemeral database.

### 12. Add request validation and rate limiting
**Complexity:** Medium · **Area:** off-chain

**Why:** The service accepts arbitrary JSON and has no abuse controls.

**Scope:** Schema validation, body-size limits, per-IP rate limiting, and
structured error responses.

**Acceptance criteria:**
- [ ] Malformed payloads return `400` with a machine-readable code.
- [ ] Oversized bodies are rejected.
- [ ] Rate limits are configurable and tested.

### 13. Add `cargo-deny` and dependency auditing to CI
**Complexity:** Medium · **Area:** CI

**Why:** A supply-chain regression in the Soroban dependency tree broke the build
once already (the `ed25519-dalek` >= 2 spec).

**Scope:** Add `deny.toml` and a CI job for licenses, advisories, and duplicate
crates. Keep `Cargo.lock` committed.

**Acceptance criteria:**
- [ ] CI fails on a new advisory or disallowed license.
- [ ] The lockfile is committed and reproducible.
- [ ] The policy is documented in `CONTRIBUTING.md`.

---

## P2 — polish

### 14. Add a canonical test-vector page for IVMS101-lite
**Complexity:** Trivial · **Area:** docs

**Why:** Implementers need a single source of truth for the hash input.

**Scope:** Add `docs/TEST-VECTORS.md` with payload → canonical string → digest
examples.

**Acceptance criteria:**
- [ ] At least three vectors, including nested and reordered keys.
- [ ] Each vector is asserted by both test suites.

### 15. Add a Testnet deployment record and smoke-test script
**Complexity:** Medium · **Area:** operations

**Why:** Reviewers and contributors need a reproducible deployed instance.

**Scope:** Commit `deployments/testnet.json` (contract IDs, commit SHA, admin,
threshold) and a script that runs the deployment checklist.

**Acceptance criteria:**
- [ ] The record is validated by CI against the schema.
- [ ] The smoke test exits non-zero on any failed check.
- [ ] The README links to the record.

### 16. Publish the SDK to npm
**Complexity:** Medium · **Area:** SDK, release

**Why:** Contributors should be able to `npm install` the package instead of
building from source.

**Scope:** Add a release workflow, provenance, and a `CHANGELOG` entry.

**Acceptance criteria:**
- [ ] A tagged release publishes `soroban-travel-rule-sdk`.
- [ ] `npm install` in a clean project works.
- [ ] The package contains `dist/` and type declarations only.
