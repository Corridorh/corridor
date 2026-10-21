# Security Policy

## Reporting a vulnerability

Please report security issues privately through GitHub's
[private vulnerability reporting](https://github.com/Corridorh/corridor/security/advisories/new).

Do not open a public issue, and do not include exploit details in a pull request
until a fix has been released.

We will acknowledge a report within 3 business days and aim to provide a
remediation plan within 10 business days. Please give us a reasonable window to
fix the issue before any public disclosure.

## Supported versions

| Version | Supported |
|---|---|
| `main` (pre-1.0) | Yes |
| Any tagged deployment older than `main` | Best effort |

Corridor has not been audited. Treat every deployment as a reference
implementation until an independent audit is published.

## Security model

Corridor assumes:

- The wrapped SEP-41 token behaves according to SEP-41.
- The configured registry contract is the one the admin intended.
- The admin key is controlled by the deploying organisation (preferably a
  multisig).
- Off-chain VASPs authenticate each other out of band. The on-chain contract
  cannot verify that a payload was actually delivered.

## Invariants

1. A metadata commitment is consumed at most once.
2. A transfer at or above the threshold cannot succeed unless both parties are
   active VASPs.
3. Only the admin can mutate the allowlist or the threshold.
4. Initialization cannot be front-run; it requires the admin's authorization.
5. A failed token transfer reverts the commitment marker.
6. No personal data is written to the ledger.

## Known limitations

- **Unaudited.** No independent audit has been completed.
- **Off-chain delivery is not proven on-chain.** The contract only proves a hash
  was supplied. Signed delivery receipts are tracked in
  [`docs/ROADMAP.md`](./docs/ROADMAP.md).
- **Encryption is a stub.** The reference off-chain service bundles the
  symmetric key with the ciphertext. It must be replaced with KMS/HSM-backed
  asymmetric encryption before real use.
- **Single-admin model.** The registry admin can add or remove VASPs and can be
  any Stellar account, including a multisig, but there is no on-chain M-of-N
  flow in v1.
- **TTL expiry.** Persistent replay markers and VASP entries can expire if they
  are never touched. Operators must run a heartbeat. See
  [`BUILD-SPEC.md`](./BUILD-SPEC.md#7-storage-lifetime-ttl).
- **Payload entropy.** A commitment to a low-entropy payload can be brute
  forced. Include a random nonce in the off-chain payload when unlinkability
  matters.
- **No upgrade path.** Contracts are immutable. A policy change requires a new
  deployment.

## Dependency and supply-chain policy

- `Cargo.lock` and both `package-lock.json` files are committed.
- Dependabot proposes updates weekly.
- `soroban-env-host` transitively requires `ed25519-dalek >= 2`; the lockfile
  pins the compatible `2.x` release. Do not regenerate the lockfile with an
  unconstrained `ed25519-dalek 3.x`.
- New dependencies require a licence and advisory review in the pull request.

## Safe deployment checklist

See [`docs/DEPLOYMENT.md`](./docs/DEPLOYMENT.md#7-deployment-checklist).
