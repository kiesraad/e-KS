#!/bin/bash

set -euo pipefail

if [ -f bin/dev ] && [ -f bin/setup ]; then
    echo "Binaries already exist. Skipping build."
    bin/setup "$@"
    exit 0
fi  

pushd development
    cargo build --release
popd

cp development/target/release/dev bin/dev
cp development/target/release/setup bin/setup

bin/setup "$@"
