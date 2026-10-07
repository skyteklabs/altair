#!/usr/bin/env bash
# Fail if any domain crate depends on infrastructure crates (ADR-0005).
set -euo pipefail
cd "$(dirname "$0")/.."

forbidden='^(axum|sqlx|tokio|hyper|tower|reqwest|utoipa|diesel|sea-orm)$'
bad=0
for manifest in domain/*/Cargo.toml; do
  crate=$(dirname "$manifest")
  deps=$(cargo tree --manifest-path "$manifest" --prefix none --edges normal,build,dev \
    | awk '{print $1}' | sort -u)
  hits=$(grep -E "$forbidden" <<<"$deps" || true)
  if [ -n "$hits" ]; then
    echo "$crate depends on infrastructure crates:" >&2
    echo "$hits" >&2
    bad=1
  fi
done
exit "$bad"
