# Assets Directory

This directory contains static assets used by the Kokoro Go CLI.

## voices.json

The `voices.json` file contains a definitive mapping of Kokoro v1.0 voice IDs to their respective names, languages, and genders.

### Why is this needed?
The downloaded Kokoro `voices.bin` model file contains 512-dimensional embeddings for 53 different voices (indices 0-52). However, `voices.bin` is a raw binary tensor and does not contain human-readable metadata (names, genders, languages). Additionally, the upstream ONNX model export from `k2-fsa/sherpa-onnx` strips this metadata.

### How was it derived?
This mapping was derived from the upstream `k2-fsa/sherpa-onnx` Python generation scripts used to build the `voices.bin` file. Specifically, it maps the speaker IDs defined in [`generate_voices_bin.py`](https://github.com/k2-fsa/sherpa-onnx/blob/master/scripts/kokoro/v1.0/generate_voices_bin.py) to their human-readable attributes.

### Updating voices.json
If new voices are added to the Kokoro model in the future:
1. Locate the updated upstream mapping (e.g., in the sherpa-onnx scripts).
2. Add the new JSON entries to this file.
3. Users simply need to copy the updated `voices.json` into their local XDG model directory (e.g., `~/.local/share/kokoro/models/v1.0/voices.json`), or rerun the `scripts/setup_models.sh` script to pull and copy the latest version.
