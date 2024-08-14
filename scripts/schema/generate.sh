#!/bin/bash

set -e

cd "$(dirname "$0")";
cd ../..

cargo run --features="generate_schema" -- --generate_schema > schema/nightly/bottom.json
