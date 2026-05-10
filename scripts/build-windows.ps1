$ErrorActionPreference = "Stop"

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$WindowsApp = Join-Path $RepoRoot "apps/windows"
$Downloads = Join-Path $RepoRoot "downloads/windows"
$BundleDir = Join-Path $WindowsApp "src-tauri/target/release/bundle/nsis"

Write-Host "Building Biz-Suite Device Bridge for Windows..."
Write-Host "Repo root: $RepoRoot"

New-Item -ItemType Directory -Force -Path $Downloads | Out-Null

Push-Location $WindowsApp
try {
  npm install
  npm run build
}
finally {
  Pop-Location
}

$Installers = Get-ChildItem -Path $BundleDir -Filter "*.exe" -Recurse -ErrorAction SilentlyContinue

if (-not $Installers -or $Installers.Count -eq 0) {
  throw "No Windows .exe installer found in $BundleDir"
}

foreach ($Installer in $Installers) {
  Copy-Item $Installer.FullName -Destination $Downloads -Force
  Write-Host "Copied: $($Installer.Name) → downloads/windows/"
}

Write-Host "Done. Windows installer output is in downloads/windows/"
