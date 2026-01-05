#!/bin/bash

set -euo pipefail

./tools/esbuild --bundle frontend/index.ts \
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
    --release \
    --bin eks
