$ErrorActionPreference = "Stop"

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$AndroidApp = Join-Path $RepoRoot "apps/android"
$Downloads = Join-Path $RepoRoot "downloads/android"

Write-Host "Building Biz-Suite Device Bridge for Android..."
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

$Apks = Get-ChildItem -Path (Join-Path $AndroidApp "app/build/outputs/apk") -Filter "*.apk" -Recurse -ErrorAction SilentlyContinue

if (-not $Apks -or $Apks.Count -eq 0) {
  throw "No Android .apk found under apps/android/app/build/outputs/apk"
}

foreach ($Apk in $Apks) {
  Copy-Item $Apk.FullName -Destination $Downloads -Force
  Write-Host "Copied: $($Apk.Name) → downloads/android/"
}

Write-Host "Done. Android APK output is in downloads/android/"
