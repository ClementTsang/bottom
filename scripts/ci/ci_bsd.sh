#!/bin/sh

# Script to be run by the `ci.yml` workflow for -BSD jobs based on the target.

set -eu

BSD_TARGET="${1:-}"

if [ -z "$BSD_TARGET" ]; then
    echo "Error: BSD target must be specified."
    exit 1
fi

if [ "$BSD_TARGET" = "x86_64-unknown-openbsd" ]; then
    pkg_add rust rust-rustfmt

    . "$HOME/.cargo/env"
    cargo fmt --all -- --check
    # Note this only tests the default features, but I think that's fine.
    # We also do not run clippy because OpenBSD tends to lag behind due to
    # it being tier 3 (see https://github.com/eza-community/eza/pull/1669).
    cargo test --no-fail-fast --locked --no-default-features -- --nocapture --quiet
else
    echo "Unsupported BSD VM target type."
    exit 1
fi
