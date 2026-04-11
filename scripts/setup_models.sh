#!/bin/bash
set -e

echo "======================================"
echo " Kokoro TTS - Model Setup Script"
echo "======================================"

# Expected SHA256 checksum for the v1.0 tarball
EXPECTED_CHECKSUM="c133d26353d776da730870dac7da07dbfc9a5e3bc80cc5e8e83ab6e823be7046"

verify_checksum() {
    local file=$1
    local expected_hash=$2
    local actual_hash=""

    echo "Verifying checksum..."
    if command -v shasum >/dev/null 2>&1; then
        actual_hash=$(shasum -a 256 "$file" | awk '{print $1}')
    elif command -v sha256sum >/dev/null 2>&1; then
        actual_hash=$(sha256sum "$file" | awk '{print $1}')
    else
        echo "Warning: Neither 'shasum' nor 'sha256sum' found. Skipping checksum validation."
        return 0
    fi

    if [ "$actual_hash" != "$expected_hash" ]; then
        echo "Error: Checksum mismatch for $file!"
        echo "  Expected: $expected_hash"
        echo "  Actual:   $actual_hash"
        return 1
    fi
    echo "Checksum passed!"
    return 0
}

# Determine XDG data directory
XDG_DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
KOKORO_MODEL_DIR="$XDG_DATA_HOME/kokoro/models/v1.0"

echo "Setting up Kokoro v1.0 (Multi-Language) at: $KOKORO_MODEL_DIR"

# Check if model already exists and prompt user
if [ -d "$KOKORO_MODEL_DIR" ]; then
    read -p "Model directory '$KOKORO_MODEL_DIR' already exists. Redownload and replace it? [y/N] " response
    case "$response" in
        [yY][eE][sS]|[yY]) 
            echo "Proceeding with redownload..."
            rm -rf "$KOKORO_MODEL_DIR"
            ;;
        *)
            echo "Skipping download."
            exit 0
            ;;
    esac
fi

mkdir -p "$KOKORO_MODEL_DIR"

URL="https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/kokoro-multi-lang-v1_0.tar.bz2"
ARCHIVE_NAME=$(basename "$URL")

echo "Downloading v1.0 from $URL..."
curl -SL -O "$URL"

# Verify the downloaded archive
if ! verify_checksum "$ARCHIVE_NAME" "$EXPECTED_CHECKSUM"; then
    echo "Aborting due to checksum failure. Cleaning up..."
    rm -f "$ARCHIVE_NAME"
    rm -rf "$KOKORO_MODEL_DIR"
    exit 1
fi

echo "Extracting $ARCHIVE_NAME..."
# Extract directly into the target directory, stripping the top-level folder from the tarball
tar xvf "$ARCHIVE_NAME" -C "$KOKORO_MODEL_DIR" --strip-components=1

echo "Cleaning up archive..."
rm -f "$ARCHIVE_NAME"

echo "Copying voices.json for v1.0..."
cp "$(dirname "$0")/../assets/voices.json" "$KOKORO_MODEL_DIR/voices.json"

echo ""
echo "Setup script finished successfully!"
