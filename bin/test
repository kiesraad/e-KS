#!/bin/bash

set -euo pipefail

export MEMORY_SERVE_QUIET=1
export SQLX_OFFLINE=true

./bin/esbuild --bundle frontend/index.ts \
    --outdir=frontend/static \
    --minify \
    --sourcemap \
    --define:IS_PRODUCTION=true \
    --loader:.woff2=file \
    --loader:.svg=file \
    --public-path=/static/

cargo build \
    --features memory-serve \
    --no-default-features \
    --features memory-serve,fixtures,dev-features \
    --release \
    --bin eks

pushd playwright
    docker compose run --rm --quiet-pull --quiet-build playwright
popd
