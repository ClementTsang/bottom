#!/bin/bash

set -e

cd "$(dirname "$0")";
cd ../..

mkdir -p schema/v$1
CARGO_TARGET_DIR="./target"cargo run --manifest-path scripts/schema_gen/Cargo.toml -- $1 > schema/v$1/bottom.json
