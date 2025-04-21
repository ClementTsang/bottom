#!/bin/bash

set -e

cd "$(dirname "$0")";
cd ../..

cargo run --bin schema --features="generate_schema" -- $1 > schema/nightly/bottom.json
