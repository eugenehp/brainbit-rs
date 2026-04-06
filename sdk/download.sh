#!/usr/bin/env bash
#
# Download and verify NeuroSDK2 native libraries from official BrainbitLLC repos.
#
# Usage:
#   ./sdk/download.sh              # download for current OS
#   ./sdk/download.sh all          # download all platforms
#   ./sdk/download.sh macos        # download macOS only
#   ./sdk/download.sh linux        # download Linux only
#   ./sdk/download.sh windows      # download Windows only
#
# Libraries are placed in sdk/lib/{macos,linux,windows}/
# Checksums are verified against sdk/checksums.sha256

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SDK_DIR="$SCRIPT_DIR"
LIB_DIR="$SDK_DIR/lib"
CHECKSUM_FILE="$SDK_DIR/checksums.sha256"

# Pinned commit SHAs (update these when upgrading SDK version)
APPLE_COMMIT="c0497ead740b"
CPP_COMMIT="c10abc74fb61"
LINUX_COMMIT="9f09ad459078"

APPLE_BASE="https://github.com/BrainbitLLC/apple_neurosdk2/raw/${APPLE_COMMIT}"
CPP_BASE="https://github.com/BrainbitLLC/neurosdk2-cpp/raw/${CPP_COMMIT}"
LINUX_BASE="https://github.com/BrainbitLLC/linux_neurosdk2/raw/${LINUX_COMMIT}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info()  { echo -e "${GREEN}[✓]${NC} $*"; }
warn()  { echo -e "${YELLOW}[!]${NC} $*"; }
error() { echo -e "${RED}[✗]${NC} $*"; }
die()   { error "$@"; exit 1; }

# ── Detect sha256sum vs shasum ────────────────────────────────────────────────

if command -v sha256sum &>/dev/null; then
    SHA256="sha256sum"
elif command -v shasum &>/dev/null; then
    SHA256="shasum -a 256"
else
    die "Neither sha256sum nor shasum found. Install coreutils."
fi

# ── Verify a single file against checksums.sha256 ────────────────────────────

verify_file() {
    local file="$1"
    local basename
    basename="$(basename "$file")"

    local expected
    expected="$(grep "  ${basename}\$" "$CHECKSUM_FILE" | awk '{print $1}')"

    if [ -z "$expected" ]; then
        die "No checksum found for ${basename} in ${CHECKSUM_FILE}"
    fi

    local actual
    actual="$($SHA256 "$file" | awk '{print $1}')"

    if [ "$actual" != "$expected" ]; then
        error "CHECKSUM MISMATCH for ${basename}!"
        error "  Expected: ${expected}"
        error "  Got:      ${actual}"
        error ""
        error "The file may have been tampered with or updated upstream."
        error "If this is an intentional SDK update, verify the new binary and"
        error "update sdk/checksums.sha256 with the new hash."
        rm -f "$file"
        return 1
    fi

    info "Checksum verified: ${basename}"
    return 0
}

# ── Download helpers ──────────────────────────────────────────────────────────

download() {
    local url="$1"
    local dest="$2"

    if [ -f "$dest" ]; then
        # Already exists — just verify
        if verify_file "$dest"; then
            info "Already downloaded: $(basename "$dest")"
            return 0
        else
            warn "Re-downloading $(basename "$dest")..."
        fi
    fi

    echo "Downloading: $(basename "$dest")"
    echo "  From: ${url}"
    mkdir -p "$(dirname "$dest")"

    if ! curl -fSL --progress-bar "$url" -o "$dest"; then
        die "Download failed: ${url}"
    fi

    verify_file "$dest"
}

# ── Platform downloaders ─────────────────────────────────────────────────────

download_macos() {
    echo ""
    echo "━━━ macOS (universal: x86_64 + arm64) ━━━"
    local dir="$LIB_DIR/macos"
    mkdir -p "$dir"
    download "${APPLE_BASE}/macos/libneurosdk2.dylib" "$dir/libneurosdk2.dylib"
}

download_linux() {
    echo ""
    echo "━━━ Linux (x86_64) ━━━"
    local dir="$LIB_DIR/linux"
    mkdir -p "$dir"
    download "${LINUX_BASE}/raw_lib/libneurosdk2.so" "$dir/libneurosdk2.so"
}

download_windows() {
    echo ""
    echo "━━━ Windows ━━━"
    local dir="$LIB_DIR/windows"
    mkdir -p "$dir"
    download "${CPP_BASE}/neurosdk2-x64.dll" "$dir/neurosdk2-x64.dll"
    download "${CPP_BASE}/neurosdk2-x32.dll" "$dir/neurosdk2-x32.dll"
}

# ── Main ──────────────────────────────────────────────────────────────────────

target="${1:-auto}"

case "$target" in
    all)
        download_macos
        download_linux
        download_windows
        ;;
    macos|darwin)
        download_macos
        ;;
    linux)
        download_linux
        ;;
    windows|win)
        download_windows
        ;;
    auto)
        case "$(uname -s)" in
            Darwin) download_macos ;;
            Linux)  download_linux ;;
            MINGW*|MSYS*|CYGWIN*) download_windows ;;
            *) die "Unknown OS: $(uname -s). Use: $0 {macos|linux|windows|all}" ;;
        esac
        ;;
    *)
        die "Unknown target: ${target}. Use: $0 {macos|linux|windows|all|auto}"
        ;;
esac

echo ""
info "Done. Libraries are in: ${LIB_DIR}/"
echo ""
echo "To use with cargo:"
case "$(uname -s)" in
    Darwin)
        echo "  export DYLD_LIBRARY_PATH=\"${LIB_DIR}/macos:\$DYLD_LIBRARY_PATH\""
        ;;
    Linux)
        echo "  export LD_LIBRARY_PATH=\"${LIB_DIR}/linux:\$LD_LIBRARY_PATH\""
        ;;
    *)
        echo "  Add ${LIB_DIR}/windows to your PATH"
        ;;
esac
echo "  cargo run"
