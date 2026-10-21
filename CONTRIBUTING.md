# Contributing to Corridor

Thanks for your interest. Corridor is a small, security-sensitive codebase; the
bar for a merge is "tests pass, behaviour is documented, and the security model
still holds."

## Code of conduct

By participating you agree to follow the [Code of Conduct](./CODE_OF_CONDUCT.md).

## Prerequisites

- Rust stable with `wasm32-unknown-unknown` (see `rust-toolchain.toml`)
- Node.js 20+
- The Stellar CLI, only if you are deploying to Testnet

## Setup

```bash
git clone https://github.com/Corridorh/corridor.git
cd corridor
rustup target add wasm32-unknown-unknown
cargo test --workspace

(cd sdk/ts && npm install)
(cd offchain/metadata-exchange && npm install)
```

## Required checks

Every pull request must keep these green:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace --target wasm32-unknown-unknown --release

(cd sdk/ts && npm run build && npm test)
(cd offchain/metadata-exchange && npm run build && npm test)
```

CI runs the same commands on Linux. Do not disable a lint or a test to make a
change pass; fix the underlying issue or explain why the rule should change.

## Making a change

1. Open or claim an issue. For anything non-trivial, describe the approach in the
   issue before writing code.
2. Create a branch from `main`:
   - `feat/<short-description>`
   - `fix/<short-description>`
   - `docs/<short-description>`
   - `refactor/<short-description>`
3. Keep commits focused. We use
   [Conventional Commits](https://www.conventionalcommits.org/):
   ```
   feat(router): add threshold getter
   fix(registry): require admin auth on initialize
   docs(spec): document TTL policy
   ```
4. Add or update tests. A bug fix needs a regression test that fails before the
   fix.
5. Update [`CHANGELOG.md`](./CHANGELOG.md) if behaviour or the ABI changed.
6. Open a pull request using the template and link the issue (`Closes #123`).

## What reviewers look for

- **Correctness:** boundary conditions, error paths, and state changes on
  failure.
- **Security:** authorization on every state-changing entry point, replay
  protection, storage lifetime, and no PII on-chain.
- **Tests:** the new behaviour is covered, and the test would fail if the change
  were reverted.
- **Documentation:** `BUILD-SPEC.md` and `docs/` reflect the code.
- **Compatibility:** no silent ABI or storage-layout change.

## Contract-specific rules

- Contracts are `#![no_std]` and must not panic on user input; return a typed
  error instead.
- Keep storage keys in a single `DataKey` enum per contract.
- Extend TTL on every live read/write of instance and persistent entries.
- Record effects before interacting with an external contract.
- Do not add a dependency without a licence and advisory review.
- `Cargo.lock` is committed. If a dependency resolution breaks the build, pin it
  and explain why in the PR.

## Drips Stellar Wave

This repository participates in the
[Drips Stellar Wave](https://docs.drips.network/wave/) program.

- Issues tagged `Stellar Wave` are eligible for the current sprint.
- Comment on the issue to apply; a maintainer will assign you.
- The issue description contains the acceptance criteria. Treat them as the
  definition of done.
- Open your PR against `main` and link the issue.
- Review time can be tight during a sprint; keep PRs small and respond to review
  comments promptly.

[`docs/ROADMAP.md`](./docs/ROADMAP.md) lists pre-scoped work with complexity and
acceptance criteria.

## Reporting a security issue

Do **not** open a public issue. Follow [`SECURITY.md`](./SECURITY.md).

## Licence

By contributing you agree that your contribution is licensed under the MIT
licence in [`LICENSE`](./LICENSE).
