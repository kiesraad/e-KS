#!/bin/bash

set -euo pipefail

export MEMORY_SERVE_QUIET=1

# check postgres is running
if pg_isready -h 127.0.0.1 -q; then
    cargo sqlx prepare -- --all-features
fi

# rust
cargo check --all-features
cargo +nightly fmt --all -- --config imports_granularity="Crate"
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features

# typescript
./tools/biome format --write ./frontend/scripts
./tools/biome check ./frontend/scripts 

# generic
./.github/workflows/check_newline.sh
