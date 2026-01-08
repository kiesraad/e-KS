#!/bin/bash

set -euo pipefail

export MEMORY_SERVE_QUIET=1

# check postgres is running
if pg_isready -h 127.0.0.1 -q; then
    cargo sqlx migrate run --database-url postgres://eks@localhost/eks
    cargo sqlx prepare --database-url postgres://eks@localhost/eks -- --all-features
fi

# rust
cargo check --all-features
cargo +nightly fmt --all -- --config imports_granularity="Crate"
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features

# typescript
./bin/biome format --write ./frontend/scripts
./bin/biome check ./frontend/scripts 

# generic
./bin/check_newline.sh
