#!/usr/bin/env bash
# Downloads the prebuilt PDFium library (https://github.com/bblanchon/pdfium-binaries) into
# vendor/pdfium/<platform>/. The release and the checksums are pinned; dms-infra-pdf loads the
# library at runtime. Usage: scripts/fetch-pdfium.sh [mac-arm64|mac-x64|win-x64|linux-x64|all]
# (default: the platform of this machine).
set -euo pipefail

RELEASE="chromium/8125" # matches the pdfium-render feature "pdfium_8125"
BASE_URL="https://github.com/bblanchon/pdfium-binaries/releases/download/${RELEASE}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

checksum_of() {
  case "$1" in
    mac-arm64) echo "4939bd935ac83ddc09a3a4f038bc43a19cea1660316a3667f26cb07a070e139d" ;;
    mac-x64) echo "8a5193535b80c7215d8932f6647ae5097465b6dba2811602c21e120c2af112b0" ;;
    win-x64) echo "9a964261760cff52570aabf038ab346d3a5034dd8b30d0d5fe8861235d356b8b" ;;
    linux-x64) echo "e1c3ce6e270cb5e0c20816b379ad9f2c1309972697c505f508de92bf5e7d6659" ;;
    *) echo "Unknown platform: $1" >&2; exit 1 ;;
  esac
}

this_platform() {
  case "$(uname -s)-$(uname -m)" in
    Darwin-arm64) echo "mac-arm64" ;;
    Darwin-x86_64) echo "mac-x64" ;;
    Linux-x86_64) echo "linux-x64" ;;
    MINGW*-x86_64 | MSYS*-x86_64 | CYGWIN*-x86_64) echo "win-x64" ;;
    *) echo "Unsupported machine: $(uname -s)-$(uname -m)" >&2; exit 1 ;;
  esac
}

sha256_of() {
  if command -v sha256sum >/dev/null; then sha256sum "$1" | cut -d' ' -f1; else shasum -a 256 "$1" | cut -d' ' -f1; fi
}

fetch() {
  local platform="$1"
  local expected archive target
  expected="$(checksum_of "$platform")"
  target="${ROOT}/vendor/pdfium/${platform}"
  archive="$(mktemp -t pdfium.XXXXXX)"
  trap 'rm -f "$archive"' RETURN
  echo "Downloading PDFium ${RELEASE} for ${platform}"
  curl --fail --location --silent --show-error --output "$archive" "${BASE_URL}/pdfium-${platform}.tgz"
  if [ "$(sha256_of "$archive")" != "$expected" ]; then
    echo "Checksum mismatch for pdfium-${platform}.tgz" >&2
    exit 1
  fi
  rm -rf "$target"
  mkdir -p "$target"
  tar -xzf "$archive" -C "$target"
  echo "${RELEASE}" > "${target}/RELEASE"
  echo "Installed into ${target}"
}

choice="${1:-$(this_platform)}"
if [ "$choice" = "all" ]; then
  for platform in mac-arm64 mac-x64 win-x64 linux-x64; do fetch "$platform"; done
else
  fetch "$choice"
fi
