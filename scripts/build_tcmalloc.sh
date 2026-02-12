#!/usr/bin/env bash
# Build Google's standalone tcmalloc and produce a self-contained static archive.
#
# Bazel's cc_library produces thin archives without transitive deps,
# so this script collects all object files and merges them into a fat
# archive that can be linked standalone.
#
# Usage:
#   ./scripts/build_tcmalloc.sh
#
# After building, set the env var for Tython's build:
#   export TCMALLOC_LIB="$(./scripts/build_tcmalloc.sh)"
#   cargo build

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TCMALLOC_DIR="$REPO_ROOT/third_party/tcmalloc"
OUT="$REPO_ROOT/third_party/libtcmalloc_bundled.a"

if [ ! -d "$TCMALLOC_DIR" ]; then
    echo "error: tcmalloc submodule not found at $TCMALLOC_DIR" >&2
    echo "Run: git submodule update --init third_party/tcmalloc" >&2
    exit 1
fi

if ! command -v bazel &>/dev/null; then
    echo "error: bazel is required to build tcmalloc" >&2
    echo "Install from: https://bazel.build/install" >&2
    exit 1
fi

cd "$TCMALLOC_DIR"
export USE_BAZEL_VERSION="${USE_BAZEL_VERSION:-7.6.0}"
bazel build //tcmalloc >&2

BAZEL_BIN="$(readlink -f "$TCMALLOC_DIR/bazel-bin")"

# Collect all object files (tcmalloc + abseil transitive deps)
OBJECTS=$(find "$BAZEL_BIN" -name "*.pic.o" -type f)

if [ -z "$OBJECTS" ]; then
    echo "error: no object files found in $BAZEL_BIN" >&2
    exit 1
fi

# Create a fat static archive
rm -f "$OUT"
echo "$OBJECTS" | xargs ar crs "$OUT"
echo "Bundled $(echo "$OBJECTS" | wc -l) objects into $OUT" >&2

echo "$OUT"
