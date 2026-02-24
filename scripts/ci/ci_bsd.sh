#!/bin/bash

# Script to be run by the `ci.yml` workflow for -BSD jobs, conditionally based on the target.
# TODO: This probably needs to work with any sh, not just bash.

set -euo pipefail

BSD_TARGET="${1:-}"

if [[ -z "$BSD_TARGET" ]]; then
    echo "Error: BSD target must be specified."
    exit 1
fi

if [[ "$BSD_TARGET" == "x86_64-unknown-freebsd" ]]; then
    pkg install -y curl bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs --output rustup.sh
    sh rustup.sh --default-toolchain stable -y

    . "$HOME/.cargo/env"
    cargo fmt --all -- --check
    # Note this only tests the default features, but I think that's fine.
    cargo test --no-fail-fast --locked -- --nocapture --quiet
    cargo clippy --all-targets --workspace -- -D warnings
elif [[ "$BSD_TARGET" == "x86_64-unknown-netbsd" ]]; then
else
    echo "Unsupported BSD target type."
    exit 1
fi
