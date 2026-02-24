#!/bin/bash

# Script to be run by the `ci.yml` workflow for -BSD jobs, conditionally based on the target.
# Usage: ci_bsd.sh <freebsd|netbsd|openbsd>

set -euo pipefail

BSD_TARGET="${1:-}"

if [[ -z "$BSD_TARGET" ]]; then
    echo "Error: BSD target must be specified."
    exit 1
fi


if [[ "$BSD_TARGET" == "x86_64-unknown-freebsd" ]]; then
elif [[ "$BSD_TARGET" == "x86_64-unknown-netbsd" ]]; then
else
    echo "Unsupported BSD target type."
    exit 1
fi