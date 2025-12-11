#!/bin/bash

# Publish crates to kellnr registry
# Usage: ./publish.sh [--dry-run]

DRY_RUN=""
if [[ "$1" == "--dry-run" ]]; then
    DRY_RUN="--dry-run"
    echo "Running in dry-run mode..."
fi

PACKAGES=(
    revm-primitives
    revm-bytecode
    revm-state
    revm-database-interface
    revm-context-interface
    revm-interpreter
    revm-precompile
    revm-database
    revm-context
    revm-handler
    revm-inspector
    revm
    revm-statetest-types
    revme
    op-revm
)

for pkg in "${PACKAGES[@]}"; do
    echo "::group::Publishing $pkg"
    cargo publish --package "$pkg" --registry kellnr --allow-dirty $DRY_RUN || {
        echo "::notice::$pkg skipped (already exists or no changes)"
    }
    echo "::endgroup::"
done

echo "✅ Done!"
