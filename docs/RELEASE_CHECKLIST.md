# Release Checklist

## Versioning

- Follow semantic versioning tags: `vMAJOR.MINOR.PATCH`.
- Update changelog/release notes summary.

## Pre-release Validation

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- Manual smoke test for GUI and CLI.

## Tag and Release

1. Ensure `dev` has been merged to `main`.
2. Create tag: `git tag vX.Y.Z`.
3. Push tag: `git push origin vX.Y.Z`.
4. Confirm `Release` workflow succeeded.
5. Verify attached release artifact and notes.

## Post-release

- Announce notable changes.
- Update roadmap for next iteration.
