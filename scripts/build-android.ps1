$ErrorActionPreference = "Stop"

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$AndroidApp = Join-Path $RepoRoot "apps/android"
$Downloads = Join-Path $RepoRoot "downloads/android"
$BuildFile = Get-Content (Join-Path $AndroidApp "app/build.gradle.kts") -Raw
$Version = if ($BuildFile -match 'versionName\s*=\s*"([^"]+)"') { $Matches[1] } else { "0.1.0" }
$VersionedName = "Biz-Suite-Device-Bridge-Android-v$Version-debug.apk"

Write-Host "Building Biz-Suite Device Bridge for Android v$Version..."
Write-Host "Repo root: $RepoRoot"

New-Item -ItemType Directory -Force -Path $Downloads | Out-Null

Push-Location $AndroidApp
try {
  if (Test-Path "./gradlew.bat") {
    ./gradlew.bat assembleDebug
  } else {
    gradle assembleDebug
  }
}
finally {
  Pop-Location
}

$Apks = Get-ChildItem -Path (Join-Path $AndroidApp "app/build/outputs/apk") -Filter "*.apk" -Recurse -ErrorAction SilentlyContinue | Sort-Object LastWriteTime -Descending

if (-not $Apks -or $Apks.Count -eq 0) {
  throw "No Android .apk found under apps/android/app/build/outputs/apk"
}

$Apk = $Apks[0]
$Destination = Join-Path $Downloads $VersionedName
Copy-Item $Apk.FullName -Destination $Destination -Force

Write-Host "Copied: $($Apk.Name) → downloads/android/$VersionedName"
Write-Host "Done. Android APK output: $Destination"
