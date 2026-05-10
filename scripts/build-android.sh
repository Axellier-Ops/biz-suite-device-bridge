#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ANDROID_APP="$REPO_ROOT/apps/android"
DOWNLOADS="$REPO_ROOT/downloads/android"

mkdir -p "$DOWNLOADS"

cd "$ANDROID_APP"
if [ -f "./gradlew" ]; then
  ./gradlew assembleDebug
else
  gradle assembleDebug
fi

mapfile -t apks < <(find "$ANDROID_APP/app/build/outputs/apk" -name "*.apk" -type f)

if [ ${#apks[@]} -eq 0 ]; then
  echo "No Android .apk found under apps/android/app/build/outputs/apk" >&2
  exit 1
fi

for apk in "${apks[@]}"; do
  cp "$apk" "$DOWNLOADS/"
  echo "Copied $(basename "$apk") to downloads/android/"
done
