#!/bin/sh

# Script to be run by the `ci.yml` workflow for -BSD jobs based on the target.

set -eu

BSD_TARGET="${1:-}"

if [ -z "$BSD_TARGET" ]; then
    echo "Error: BSD target must be specified."
    exit 1
fi

if [ "$BSD_TARGET" = "x86_64-unknown-freebsd" ]; then
    pkg install -y curl bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs --output rustup.sh
    sh rustup.sh --default-toolchain stable -y

    . "$HOME/.cargo/env"
    cargo test --no-fail-fast --locked -- --nocapture --quiet
elif [ "$BSD_TARGET" = "x86_64-unknown-netbsd" ]; then
    /usr/sbin/pkg_add -u curl bash mozilla-rootcerts-openssl
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs --output rustup.sh
    sh rustup.sh --default-toolchain stable -y

    . "$HOME/.cargo/env"
    # TODO: Support default features eventually?
    cargo test --no-fail-fast --locked --no-default-features -- --nocapture --quiet
else
    echo "Unsupported BSD target type."
    exit 1
fi
