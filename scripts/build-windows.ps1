$ErrorActionPreference = "Stop"

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$WindowsApp = Join-Path $RepoRoot "apps/windows"
$Downloads = Join-Path $RepoRoot "downloads/windows"
$BundleDir = Join-Path $WindowsApp "src-tauri/target/release/bundle/nsis"
$PackageJson = Get-Content (Join-Path $WindowsApp "package.json") | ConvertFrom-Json
$Version = $PackageJson.version
$VersionedName = "Biz-Suite-Device-Bridge-Windows-v$Version.exe"

Write-Host "Building Biz-Suite Device Bridge for Windows v$Version..."
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

$Installers = Get-ChildItem -Path $BundleDir -Filter "*.exe" -Recurse -ErrorAction SilentlyContinue | Sort-Object LastWriteTime -Descending

if (-not $Installers -or $Installers.Count -eq 0) {
  throw "No Windows .exe installer found in $BundleDir"
}

$Installer = $Installers[0]
$Destination = Join-Path $Downloads $VersionedName
Copy-Item $Installer.FullName -Destination $Destination -Force

Write-Host "Copied: $($Installer.Name) → downloads/windows/$VersionedName"
Write-Host "Done. Windows installer output: $Destination"
