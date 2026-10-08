#!/usr/bin/env bash
# Fail if any domain crate depends on infrastructure crates (ADR-0005).
# This list is the single source of truth for what counts as infrastructure.
# Each name also bans every crate named <name>-* or <name>_* (sqlx-core, tokio-util,
# diesel_migrations, ...). This over-matches on purpose: a harmless crate that shares
# a prefix (smol_str, matched by smol) is flagged too. Aliasing does not help (cargo tree
# prints the real package name); drop the dependency or narrow the pattern below.
# Some entries are prefixes, not crates: aws-sdk matches aws-sdk-s3 and friends, which is
# why aws-config and aws-smithy are listed too.
# Dev-dependencies are checked as well, so a test-only tokio fails CI too.
# Target-specific dependencies are checked on all platforms (--target all).
set -euo pipefail
cd "$(dirname "$0")/.."

forbidden='^(axum|sqlx|tokio|hyper|tower|reqwest|utoipa|diesel|sea-orm|actix|rusqlite|postgres|redis|async-std|smol|tonic|mongodb|rdkafka|aws-sdk|aws-config|aws-smithy|lapin)([-_].*)?$'
bad=0
for manifest in domain/*/Cargo.toml; do
  crate=$(dirname "$manifest")
  deps=$(cargo tree --manifest-path "$manifest" --prefix none --edges normal,build,dev --all-features --target all \
    | awk '{print $1}' | sort -u)
  hits=$(grep -E "$forbidden" <<<"$deps" || true)
  if [ -n "$hits" ]; then
    echo "$crate depends on infrastructure crates:" >&2
    echo "$hits" >&2
    bad=1
  fi
done
exit "$bad"
