## Summary

<!-- What changed, and why? Link the issue this closes. -->

Closes #

## Type of change

- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] Documentation
- [ ] Refactor / internal cleanup
- [ ] Breaking change (contract ABI, storage layout, or behaviour)

## Checklist

- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `cargo test --workspace` passes
- [ ] `cargo build --workspace --target wasm32-unknown-unknown --release` passes
- [ ] Added or updated tests that cover the change
- [ ] Updated `CHANGELOG.md` when behaviour or ABI changed
- [ ] Updated documentation (`README.md`, `docs/`, `BUILD-SPEC.md`) when needed

## Security notes

<!--
Describe any effect on the security model: authorization, replay protection,
storage lifetime/TTL, asset handling, or PII exposure. Write "None" if not
applicable.
-->

## Verification

<!-- Commands run and their results, plus any manual testnet verification. -->
