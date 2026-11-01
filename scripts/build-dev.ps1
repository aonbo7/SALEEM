# SALEEM Development Build & Verification Script
$ErrorActionPreference = "Stop"

$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
$machinePath = [Environment]::GetEnvironmentVariable("Path", "Machine")
$env:PATH = "$env:USERPROFILE\.cargo\bin;$userPath;$machinePath"

Write-Host "=== [1/3] Running Frontend TypeScript Checks ===" -ForegroundColor Cyan
Push-Location "$PSScriptRoot\..\apps\desktop"
npm.cmd run typecheck
Pop-Location

Write-Host "=== [2/3] Running Rust Workspace Unit & Safety Tests ===" -ForegroundColor Cyan
Push-Location "$PSScriptRoot\.."
cargo test --workspace
Pop-Location

Write-Host "=== [3/3] Building Debug Workspace & Native DLLs ===" -ForegroundColor Cyan
Push-Location "$PSScriptRoot\.."
cargo build --workspace
Pop-Location
