#!/usr/bin/env bash
set -e

# Required temporary storage
TMP_DIR=$(mktemp -d /tmp/dedup.XXXXXX)

cleanup() {
    rm -rf "${TMP_DIR}"
}

trap cleanup EXIT SIGINT SIGTERM

# Help documentation
usage() {
    echo "Usage: $0 <input_glob> <output_directory>"
    echo "This script checks if the given parameters are directories."
    echo "  <input_globs>\tcomma separated list of globs to deduplicate"
    echo "  <output_dir>\tdirectory to save the deuplicated files to"
    echo "Example: $0 input_dir/\*.txt output_dir/"
    exit 1
}

# Check for help option
if [[ "$1" == "--help" ]]; then
    usage
fi

# Check for correct number of arguments
if [ "$#" -ne 2 ]; then
    echo "Error: You must enter exactly two parameters."
    usage
fi

# Validate inputs
validate_glob() {
    local pattern="$1"
    local files=( $pattern )

    # Check if the expansion returns anything
    if [ ${#files[@]} -eq 0 ]; then
        echo "No files match the glob '$pattern'."
        echo "Please provide a valid glob pattern."
        return 1 # Return 1 for error state
    else
        return 0 # Return 0 for success state
    fi
}

validate_directory() {
    if [ ! -d "$1" ]; then
        echo "Error: '$1' is not a valid directory."
        exit 2
    fi
}

log() {
    echo -e "\033[32m${1}\033[0m"
}

validate_directory "$2"

IFS=',' read -ra INPUT_GLOBS <<< "$1"
for i in "${INPUT_GLOBS[@]}"; do
    validate_glob "$i"
done
OUTPUT_DIR="$2"

log "Hashing documents..."
for GLOB in "${INPUT_GLOBS[@]}"; do
    GLOB="${GLOB/#\~/$HOME}"
    GLOB="${GLOB/#\./$PWD}"
    ./target/release/hash_documents "$GLOB" "$TMP_DIR" > /dev/null &
done

wait

log "\nIntermediate hash files (first char of filename is the shard):"
ls "$TMP_DIR/"

SHARDS=($(ls "$TMP_DIR/" | sed 's/^\(.\).*/\1/' | sort -u))

log "\nDeduping files based on their hashes..."

for SHARD in "${SHARDS[@]}"; do
    ./target/release/dedup_hashes "$TMP_DIR/${SHARD}_*" "$OUTPUT_DIR/$SHARD.csv" > /dev/null &
done

wait

log "\nUniques (first char of the filename is the shard):"
ls $OUTPUT_DIR/*.csv

echo ""
log "Run the following command to get uniques: ls $OUTPUT_DIR/*.csv | xargs cat"