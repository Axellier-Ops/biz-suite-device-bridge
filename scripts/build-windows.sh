#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOWNLOADS="$REPO_ROOT/downloads/windows"
BUNDLE_DIR="$REPO_ROOT/src-tauri/target/release/bundle/nsis"
VERSION="$(node -p "require('$REPO_ROOT/package.json').version")"
VERSIONED_NAME="Biz-Suite-Cloud-POS-Windows-v$VERSION.exe"

mkdir -p "$DOWNLOADS"
node "$REPO_ROOT/scripts/prepare-windows-icon.cjs" "$REPO_ROOT"

cd "$REPO_ROOT"
npm install
npm run build

shopt -s nullglob globstar
installers=("$BUNDLE_DIR"/**/*.exe "$BUNDLE_DIR"/*.exe)

if [ ${#installers[@]} -eq 0 ]; then
  echo "No Windows .exe installer found in $BUNDLE_DIR" >&2
  exit 1
fi

latest_installer="$(ls -t "${installers[@]}" | head -n 1)"
cp "$latest_installer" "$DOWNLOADS/$VERSIONED_NAME"
echo "Copied $(basename "$latest_installer") to downloads/windows/$VERSIONED_NAME"
