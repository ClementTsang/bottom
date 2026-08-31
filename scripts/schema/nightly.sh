#!/bin/bash

set -e

cd "$(dirname "$0")";
cd ../..

cargo run --manifest-path scripts/schema_gen/Cargo.toml > schema/nightly/bottom.json
