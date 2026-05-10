$ErrorActionPreference = "Stop"

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$Downloads = Join-Path $RepoRoot "downloads"

New-Item -ItemType Directory -Force -Path $Downloads | Out-Null

Write-Host "Building all Biz-Suite Device Bridge artifacts..."

& (Join-Path $PSScriptRoot "build-windows.ps1")
& (Join-Path $PSScriptRoot "build-android.ps1")

Write-Host "All build outputs copied into downloads/."
