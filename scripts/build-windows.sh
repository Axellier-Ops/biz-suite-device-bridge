#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WINDOWS_APP="$REPO_ROOT/apps/windows"
DOWNLOADS="$REPO_ROOT/downloads/windows"
BUNDLE_DIR="$WINDOWS_APP/src-tauri/target/release/bundle/nsis"
VERSION="$(node -p "require('$WINDOWS_APP/package.json').version")"
VERSIONED_NAME="biz-suite-device-bridge-windows-v$VERSION.exe"

mkdir -p "$DOWNLOADS"

cd "$WINDOWS_APP"
npm install
npm run build

shopt -s nullglob
installers=("$BUNDLE_DIR"/**/*.exe "$BUNDLE_DIR"/*.exe)

if [ ${#installers[@]} -eq 0 ]; then
  echo "No Windows .exe installer found in $BUNDLE_DIR" >&2
  exit 1
fi

rm -f "$DOWNLOADS"/*.exe
latest_installer="$(ls -t "${installers[@]}" | head -n 1)"
cp "$latest_installer" "$DOWNLOADS/$VERSIONED_NAME"
echo "Copied $(basename "$latest_installer") to downloads/windows/$VERSIONED_NAME"
