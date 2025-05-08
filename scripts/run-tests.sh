# ./run-tests --help

#!/bin/bash
set -e

# Version for the execution spec tests
VERSION="v4.4.0"
# Version for the EOF spec tests, it is currently upgrading to eof devnet-1 so we will use devnet-0 suite.
EOF_VERSION="v4.4.0"

# Directories
FIXTURES_DIR="test-fixtures"
STABLE_DIR="$FIXTURES_DIR/stable"
DEVELOP_DIR="$FIXTURES_DIR/develop"
EOF_DIR="$FIXTURES_DIR/eof"

# URL and filenames
FIXTURES_URL="https://github.com/ethereum/execution-spec-tests/releases/download"
STABLE_TAR="fixtures_stable.tar.gz"
DEVELOP_TAR="fixtures_develop.tar.gz"
EOF_TAR="fixtures_eip7692.tar.gz"

# Print usage information and exit
usage() {
    echo "Usage: $0 [clean] [runner] [profile] [target]"
    echo ""
    echo "Arguments (after optional 'clean'):"
    echo "  runner   (Optional) Rust runner command. Must be either 'cargo' or 'cross'. Defaults to 'cargo'."
    echo "  profile  (Optional) Rust profile to use. Defaults to 'debug' if not provided."
    echo "  target   (Optional) Rust target. Only used if provided."
    echo ""
    echo "Examples:"
    echo "  $0"
    echo "      Uses runner 'cargo', profile 'debug', and no target."
    echo ""
    echo "  $0 release"
    echo "      Uses runner 'cargo', profile 'release', and no target."
    echo ""
    echo "  $0 release x86-win"
    echo "      Uses runner 'cargo', profile 'release', with target 'x86-win'."
    echo ""
    echo "  $0 clean"
    echo "      Cleans fixtures then uses runner 'cargo', profile 'debug', and no target."
    echo ""
    echo "  $0 clean cross release x86-win"
    echo "      Cleans fixtures then uses runner 'cross', profile 'release', and target 'x86-win'."
    exit 1
}

# Check for help flag in any argument.
for arg in "$@"; do
    if [ "$arg" = "-h" ] || [ "$arg" = "--help" ]; then
        usage
    fi
done

# Deletes the test fixture directory
clean() {
    echo "Cleaning test fixtures..."
    rm -rf "$FIXTURES_DIR"
    echo "Cleaned test fixtures directory."
}

# Check if all required fixture directories exist
check_fixtures() {
    if [ -d "$STABLE_DIR" ] && [ -d "$DEVELOP_DIR" ] && [ -d "$EOF_DIR" ]; then
        return 0
    else
        return 1
    fi
}

# Download and extract a single fixture
# Arguments: target directory, tar file name, label for logging
download_and_extract() {
    local target_dir="$1"
    local tar_file="$2"
    local label="$3"
    local version="$4"

    echo "Downloading ${label} fixtures..."
    curl -L "${FIXTURES_URL}/${version}/${tar_file}" -o "${FIXTURES_DIR}/${tar_file}"
    echo "Extracting ${label} fixtures..."
     # strip-components=1 removes the first top level directory from the flepath
     # This is needed because when we extract the tar, it is placed under an
     # unnecessary "fixtures/" directory.
    tar -xzf "${FIXTURES_DIR}/${tar_file}" --strip-components=1 -C "$target_dir"
}

# Download all fixtures
download_fixtures() {
    echo "Creating fixtures directory structure..."
    mkdir -p "$STABLE_DIR" "$DEVELOP_DIR" "$EOF_DIR"

    download_and_extract "$STABLE_DIR" "$STABLE_TAR" "stable" "$VERSION"
    download_and_extract "$DEVELOP_DIR" "$DEVELOP_TAR" "develop" "$VERSION"
    download_and_extract "$EOF_DIR" "$EOF_TAR" "EOF" "$EOF_VERSION"

    echo "Cleaning up tar files..."
    rm "${FIXTURES_DIR}/${STABLE_TAR}" "${FIXTURES_DIR}/${DEVELOP_TAR}" "${FIXTURES_DIR}/${EOF_TAR}"
    echo "Fixtures download and extraction complete."
}

# Build Cargo options based on provided profile and target.
# For the profile:
#   - "debug" is the default (no extra option needed)
#   - "release" adds "--release"
#   - Any other value adds "--profile <profile>"
# For the target:
#   - If provided, add "--target <target>"
build_cargo_options() {
    CARGO_OPTS=""

    if [ "$RUST_PROFILE" = "release" ]; then
        CARGO_OPTS="--release"
    elif [ "$RUST_PROFILE" != "debug" ]; then
        CARGO_OPTS="--profile $RUST_PROFILE"
    fi

    if [ -n "$RUST_TARGET" ]; then
        CARGO_OPTS="$CARGO_OPTS --target $RUST_TARGET"
    fi
}

# Run tests for each set of fixtures using the chosen runner.
run_tests() {
    echo "Running stable statetests..."
    $RUST_RUNNER run $CARGO_OPTS -p revme -- statetest "$STABLE_DIR/state_tests"

    echo "Running develop statetests..."
    $RUST_RUNNER run $CARGO_OPTS -p revme -- statetest "$DEVELOP_DIR/state_tests"

    echo "Skipping EOF statetests..."
    # $RUST_RUNNER run $CARGO_OPTS -p revme -- statetest "$EOF_DIR/state_tests"

    echo "Skipping EOF validation tests..."
    # $RUST_RUNNER run $CARGO_OPTS -p revme -- eof-validation "$EOF_DIR/eof_tests"
}

##############################
# Main logic

# If the first argument is "clean", perform cleaning and download fixtures.
if [ "$1" = "clean" ]; then
    clean
    download_fixtures
    shift
else
    if check_fixtures; then
        echo "Using existing test fixtures."
    else
        echo "Test fixtures not found. Downloading..."
        download_fixtures
    fi
fi

# Argument parsing for runner, profile, target.
# Expected order (after optional clean): [runner] [profile] [target]
# If the first argument is "cargo" or "cross", then it is the runner.
# Otherwise, runner defaults to "cargo", and the arguments are profile and target.
if [ "$#" -eq 0 ]; then
    RUST_RUNNER="cargo"
    RUST_PROFILE="debug"
    RUST_TARGET=""
elif [ "$#" -eq 1 ]; then
    if [ "$1" = "cargo" ] || [ "$1" = "cross" ]; then
        RUST_RUNNER="$1"
        RUST_PROFILE="debug"
        RUST_TARGET=""
    else
        RUST_RUNNER="cargo"
        RUST_PROFILE="$1"
        RUST_TARGET=""
    fi
elif [ "$#" -eq 2 ]; then
    if [ "$1" = "cargo" ] || [ "$1" = "cross" ]; then
        RUST_RUNNER="$1"
        RUST_PROFILE="$2"
        RUST_TARGET=""
    else
        RUST_RUNNER="cargo"
        RUST_PROFILE="$1"
        RUST_TARGET="$2"
    fi
elif [ "$#" -eq 3 ]; then
    if [ "$1" = "cargo" ] || [ "$1" = "cross" ]; then
        RUST_RUNNER="$1"
        RUST_PROFILE="$2"
        RUST_TARGET="$3"
    else
        usage
    fi
else
    usage
fi

build_cargo_options
run_tests

