#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ANDROID_APP="$REPO_ROOT/apps/android"
DOWNLOADS="$REPO_ROOT/downloads/android"
VERSION="$(grep -oE 'versionName = "[^"]+"' "$ANDROID_APP/app/build.gradle.kts" | head -n 1 | sed -E 's/versionName = "([^"]+)"/\1/')"
VERSION="${VERSION:-0.1.0}"
VERSIONED_NAME="Biz-Suite-Device-Bridge-Android-v$VERSION-debug.apk"

mkdir -p "$DOWNLOADS"

cd "$ANDROID_APP"
if [ -f "./gradlew" ]; then
  ./gradlew assembleDebug
else
  gradle assembleDebug
fi

mapfile -t apks < <(find "$ANDROID_APP/app/build/outputs/apk" -name "*.apk" -type f | sort -r)

if [ ${#apks[@]} -eq 0 ]; then
  echo "No Android .apk found under apps/android/app/build/outputs/apk" >&2
  exit 1
fi

cp "${apks[0]}" "$DOWNLOADS/$VERSIONED_NAME"
echo "Copied $(basename "${apks[0]}") to downloads/android/$VERSIONED_NAME"
