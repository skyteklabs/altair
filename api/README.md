# api

Rust workspace for the Palmy API.

## Layout

- `domain/*`: domain crates. Pure rules, **no infrastructure dependencies**. See ADR-0005. CI runs `scripts/check-domain-deps.sh`; the banned list is its `banned` array and the matching rules are in its header.
- `services/*`: service crates (not created yet) that wire domain crates to HTTP and storage. Add `"services/*"` to `members` in `Cargo.toml` when the first one lands.

## Commands

The Rust toolchain is pinned in `rust-toolchain.toml`; rustup installs it on first use.

```sh
cargo test --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
./scripts/check-domain-deps.sh   # domain crates stay free of infrastructure
```

## CI

`.github/workflows/api.yml` runs the commands above on pull requests and on `main`, only when `api/**` or the workflow file changes (ADR-0003), and on demand via `workflow_dispatch`.
