#!/usr/bin/env bash
set -eo pipefail

# Usage: ./scripts/run-tests.sh --help

# Version for the execution spec tests
MAIN_VERSION="v5.3.0"
DEVNET_VERSION="fusaka-devnet-5@v2.1.0"

### Directories ###
FIXTURES_DIR="test-fixtures"

MAIN_DIR="$FIXTURES_DIR/main"
MAIN_STABLE_DIR="$MAIN_DIR/stable"
MAIN_DEVELOP_DIR="$MAIN_DIR/develop"

DEVNET_DIR="$FIXTURES_DIR/devnet"
DEVNET_DEVELOP_DIR="$DEVNET_DIR/develop"

LEGACY_DIR="$FIXTURES_DIR/legacytests" 

### URL and filenames ###
FIXTURES_URL="https://github.com/ethereum/execution-spec-tests/releases/download"

MAIN_STABLE_TAR="fixtures_stable.tar.gz"
MAIN_DEVELOP_TAR="fixtures_develop.tar.gz"

DEVNET_TAR="fixtures_fusaka-devnet-5.tar.gz"

LEGACY_REPO_URL="https://github.com/ethereum/legacytests.git"

# Print usage information and exit
usage() {
    echo "Usage: $0 [clean] [--keep-going] [runner] [profile] [target]"
    echo ""
    echo "Flags (can be specified before or after 'clean'):"
    echo "  --keep-going  Continue running tests even after failures."
    echo ""
    echo "Arguments (after optional 'clean' and '--keep-going'):"
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
    echo "  $0 --keep-going release"
    echo "      Uses runner 'cargo', profile 'release', and keeps going on test failures."
    echo ""
    echo "  $0 release x86-win"
    echo "      Uses runner 'cargo', profile 'release', with target 'x86-win'."
    echo ""
    echo "  $0 clean"
    echo "      Cleans fixtures then uses runner 'cargo', profile 'debug', and no target."
    echo ""
    echo "  $0 clean --keep-going cross release x86-win"
    echo "      Cleans fixtures then uses runner 'cross', profile 'release', target 'x86-win', and keeps going on failures."
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
    if [ -d "$MAIN_STABLE_DIR" ] && [ -d "$MAIN_DEVELOP_DIR" ] && [ -d "$DEVNET_DIR" ] && [ -d "$LEGACY_DIR" ]; then
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
    # Use -fsSL to fail on HTTP errors; add small retry for transient network issues
    curl -fsSL --retry 3 --retry-delay 2 "${FIXTURES_URL}/${version}/${tar_file}" -o "${FIXTURES_DIR}/${tar_file}"
    echo "Extracting ${label} fixtures..."
     # strip-components=1 removes the first top level directory from the flepath
     # This is needed because when we extract the tar, it is placed under an
     # unnecessary "fixtures/" directory.
    tar -xzf "${FIXTURES_DIR}/${tar_file}" --strip-components=1 -C "$target_dir"

    # Remove the tar file
    rm "${FIXTURES_DIR}/${tar_file}"

    # remove all unused folders
    find "$target_dir" -depth -name blockchain_tests_engine -exec rm -rf {} \;
    find "$target_dir" -depth -name blockchain_tests_engine_x -exec rm -rf {} \;
    find "$target_dir" -depth -name blockchain_tests_sync -exec rm -rf {} \;
}

# Download all fixtures
download_fixtures() {
    echo "Creating fixtures directory structure..."
    mkdir -p "$MAIN_STABLE_DIR" "$MAIN_DEVELOP_DIR" "$DEVNET_DIR" "$LEGACY_DIR"
    
    download_and_extract "$MAIN_STABLE_DIR" "$MAIN_STABLE_TAR" "main stable" "$MAIN_VERSION"
    download_and_extract "$MAIN_DEVELOP_DIR" "$MAIN_DEVELOP_TAR" "main develop" "$MAIN_VERSION"
    download_and_extract "$DEVNET_DIR" "$DEVNET_TAR" "devnet" "$DEVNET_VERSION"

    # Clone legacytests repository
    echo "Cloning legacytests repository..."
    git clone --depth 1 "$LEGACY_REPO_URL" "$LEGACY_DIR"
    
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
    echo "Running main stable statetests..."
    $RUST_RUNNER run $CARGO_OPTS -p revme -- statetest $KEEP_GOING_FLAG "$MAIN_STABLE_DIR/state_tests"

    echo "Running main develop statetests..."
    $RUST_RUNNER run $CARGO_OPTS -p revme -- statetest $KEEP_GOING_FLAG "$MAIN_DEVELOP_DIR/state_tests"

    echo "Running devnet statetests..."
    $RUST_RUNNER run $CARGO_OPTS -p revme -- statetest $KEEP_GOING_FLAG "$DEVNET_DIR/state_tests"

    echo "Running legacy Cancun tests..."
    $RUST_RUNNER run $CARGO_OPTS -p revme -- statetest $KEEP_GOING_FLAG "$LEGACY_DIR/Cancun/GeneralStateTests"

    echo "Running legacy Constantinople tests..."
    $RUST_RUNNER run $CARGO_OPTS -p revme -- statetest $KEEP_GOING_FLAG "$LEGACY_DIR/Constantinople/GeneralStateTests"

    echo "Running main develop blockchain tests..."
    $RUST_RUNNER run $CARGO_OPTS -p revme -- btest $KEEP_GOING_FLAG "$MAIN_DEVELOP_DIR/blockchain_tests"

    echo "Running main stable blockchain tests..."
    $RUST_RUNNER run $CARGO_OPTS -p revme -- btest $KEEP_GOING_FLAG "$MAIN_STABLE_DIR/blockchain_tests"
}

##############################
# Main logic

# Initialize flags
KEEP_GOING_FLAG=""
DID_CLEAN=false

# Process "clean" and "--keep-going" flags
while true; do
    if [ "$1" = "clean" ]; then
        clean
        download_fixtures
        DID_CLEAN=true
        shift
    elif [ "$1" = "--keep-going" ]; then
        KEEP_GOING_FLAG="--keep-going"
        shift
    else
        break
    fi
done

# If no clean was specified, check for existing fixtures
if [ "$DID_CLEAN" = false ]; then
    if check_fixtures; then
        echo "Using existing test fixtures."
    else
        echo "Test fixtures not found. Downloading..."
        download_fixtures
    fi
fi

# Argument parsing for runner, profile, target.
# Expected order (after optional clean and --keep-going): [runner] [profile] [target]
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

