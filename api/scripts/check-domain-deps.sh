#!/usr/bin/env bash
# Fail if any domain crate depends on infrastructure crates (ADR-0005).
# The banned array below is the single source of truth for what counts as infrastructure.
# Each name also bans every crate named <name>-* or <name>_* (sqlx-core, tokio-util,
# diesel_migrations, ...). This over-matches on purpose: a harmless crate that shares
# a prefix (smol_str, matched by smol) is flagged too. Aliasing does not help (cargo tree
# prints the real package name); drop the dependency or narrow the forbidden regex below.
# aws-sdk and aws-smithy are prefixes (aws-sdk-s3, aws-smithy-runtime);
# aws-config is under neither, so it is listed too.
# Dev-dependencies are checked as well, so a test-only tokio fails CI too.
# Target-specific dependencies are checked on all platforms (--target all).
set -euo pipefail
cd "$(dirname "$0")/.."

banned=(
  axum
  sqlx
  tokio
  hyper
  tower
  reqwest
  utoipa
  diesel
  sea-orm
  actix
  rusqlite
  postgres
  redis
  async-std
  smol
  tonic
  mongodb
  rdkafka
  aws-sdk
  aws-config
  aws-smithy
  lapin
)
# Join with | for the regex alternation (IFS applies to "${banned[*]}" only inside the subshell).
forbidden="^($(IFS='|'; echo "${banned[*]}"))([-_].*)?\$"
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
# time's std and local-offset features read the system clock or the local
# timezone, which domain crates must not do: "today" is always an input (ADR-0013).
# Feature edges are checked the same way as dependency edges, so transitive
# enablement (e.g. formatting pulling in std) is caught too.
for manifest in domain/*/Cargo.toml; do
  crate=$(dirname "$manifest")
  clock_features=$(cargo tree --manifest-path "$manifest" -e features,normal,build,dev --all-features --target all \
    -i time --prefix none 2>/dev/null \
    | grep -E '^time feature "(std|local-offset)"' | sort -u || true)
  if [ -n "$clock_features" ]; then
    echo "$crate enables time features that read the clock:" >&2
    echo "$clock_features" >&2
    bad=1
  fi
done
exit "$bad"
