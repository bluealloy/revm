#!/usr/bin/env bash
set -eo pipefail

# Usage: ./scripts/set-version.sh <version>
#
# Sets all publishable workspace crates to the given version
# using `release-plz set-version`.

if [ -z "$1" ]; then
    echo "Usage: $0 <version>" >&2
    echo "Example: $0 30.0.0" >&2
    exit 1
fi
VERSION="$1"

if ! command -v release-plz >/dev/null; then
    echo "release-plz is not installed. Install it with: cargo install release-plz" >&2
    exit 1
fi

if ! command -v jq >/dev/null; then
    echo "jq is not installed." >&2
    exit 1
fi

cd "$(dirname "$0")/.."

# All workspace crates except non-publishable ones (examples).
CRATES=$(cargo metadata --no-deps --format-version 1 |
    jq -r '.packages[] | select(.publish != []) | .name')

ARGS=()
for crate in $CRATES; do
    ARGS+=("${crate}@${VERSION}")
done

echo "Setting version $VERSION for:"
printf '  %s\n' "${ARGS[@]}"

release-plz set-version "${ARGS[@]}"
