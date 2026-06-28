#!/bin/bash

set -e

cd "$(dirname "$0")";
cd ../..

cargo run --manifest-path tools/schema_gen/Cargo.toml -- $1 > schema/v$1/bottom.json
