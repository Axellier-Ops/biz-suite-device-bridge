#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WINDOWS_APP="$REPO_ROOT/apps/windows"
DOWNLOADS="$REPO_ROOT/downloads/windows"
BUNDLE_DIR="$WINDOWS_APP/src-tauri/target/release/bundle/nsis"

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

for installer in "${installers[@]}"; do
  cp "$installer" "$DOWNLOADS/"
  echo "Copied $(basename "$installer") to downloads/windows/"
done
