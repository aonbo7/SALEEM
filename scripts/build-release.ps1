# SALEEM Organized Release Build & Packaging Script
$ErrorActionPreference = "Stop"

$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
$machinePath = [Environment]::GetEnvironmentVariable("Path", "Machine")
$env:PATH = "$env:USERPROFILE\.cargo\bin;$userPath;$machinePath"

$repoRoot = (Resolve-Path "$PSScriptRoot\..").Path
$envFile = Join-Path $repoRoot ".env"
if (Test-Path $envFile) {
    Get-Content $envFile | ForEach-Object {
        $line = $_.Trim()
        if ($line -and -not $line.StartsWith("#") -and $line.Contains("=")) {
            $parts = $line.Split("=", 2)
            [System.Environment]::SetEnvironmentVariable($parts[0].Trim(), $parts[1].Trim(), "Process")
        }
    }
}
$releaseDir = Join-Path $repoRoot "release"
$nativeDir = Join-Path $releaseDir "Native"
$resourcesDir = Join-Path $releaseDir "Resources"
$licensesDir = Join-Path $releaseDir "licenses"

Write-Host "=== [1/4] Building Frontend Production Assets ===" -ForegroundColor Cyan
Push-Location "$repoRoot\apps\desktop"
npm.cmd run build
Pop-Location

Write-Host "=== [2/4] Building Rust Workspace & Native Engine DLLs (Release) ===" -ForegroundColor Cyan
Push-Location $repoRoot
if (-not ((Test-Path "target\release\saleem_disk_engine.dll") -and (Test-Path "target\release\saleem_cleanup_engine.dll") -and (Test-Path "target\release\saleem_sentinel.dll") -and (Test-Path "target\release\saleem_memory_engine.dll"))) {
    cargo build --release -p saleem_disk_engine -p saleem_cleanup_engine -p saleem_sentinel -p saleem_memory_engine
} else {
    cargo build --release -p saleem_memory_engine
}
cargo build --release -p saleem_desktop
Pop-Location

Write-Host "=== [3/4] Assembling Organized release/ Directory ===" -ForegroundColor Cyan
if (Test-Path $releaseDir) {
    Remove-Item -Recurse -Force $releaseDir
}
New-Item -ItemType Directory -Path $releaseDir, $nativeDir, $resourcesDir, $licensesDir | Out-Null

$targetRelease = Join-Path $repoRoot "target\release"

Copy-Item (Join-Path $targetRelease "SALEEM.exe") (Join-Path $releaseDir "SALEEM.exe") -Force
if (Test-Path (Join-Path $targetRelease "WebView2Loader.dll")) {
    Copy-Item (Join-Path $targetRelease "WebView2Loader.dll") (Join-Path $releaseDir "WebView2Loader.dll") -Force
}
Copy-Item (Join-Path $targetRelease "saleem_disk_engine.dll") (Join-Path $nativeDir "SALEEM.DiskEngine.dll") -Force
Copy-Item (Join-Path $targetRelease "saleem_cleanup_engine.dll") (Join-Path $nativeDir "SALEEM.CleanupEngine.dll") -Force
Copy-Item (Join-Path $targetRelease "saleem_sentinel.dll") (Join-Path $nativeDir "SALEEM.Sentinel.dll") -Force
Copy-Item (Join-Path $targetRelease "saleem_memory_engine.dll") (Join-Path $nativeDir "SALEEM.MemoryEngine.dll") -Force
Copy-Item (Join-Path $repoRoot "LICENSE") (Join-Path $licensesDir "LICENSE-SALEEM.txt") -Force

if (Test-Path (Join-Path $repoRoot "rules")) {
    Copy-Item -Recurse (Join-Path $repoRoot "rules") (Join-Path $resourcesDir "rules") -Force
}

Write-Host "=== [3.5/4] Building Official Windows NSIS Installer (SALEEM-Setup.exe) ===" -ForegroundColor Cyan
Push-Location "$repoRoot\apps\desktop"
npx.cmd tauri bundle --bundles nsis
Pop-Location

$nsisDir = Join-Path $targetRelease "bundle\nsis"
if (Test-Path $nsisDir) {
    $setupExe = Get-ChildItem -Path $nsisDir -Filter "*-setup.exe" | Sort-Object LastWriteTime -Descending | Select-Object -First 1
    if ($setupExe) {
        Copy-Item $setupExe.FullName (Join-Path $releaseDir $setupExe.Name) -Force
        Copy-Item $setupExe.FullName (Join-Path $releaseDir "SALEEM-Setup.exe") -Force
    }
}

Write-Host "=== [4/4] Generating SHA256SUMS.txt ===" -ForegroundColor Cyan
$hashLines = @()
$artifacts = Get-ChildItem -Path $releaseDir -Recurse -File | Where-Object { $_.Name -ne "SHA256SUMS.txt" }
foreach ($file in $artifacts) {
    $hash = (Get-FileHash -Path $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    $relPath = $file.FullName.Substring($releaseDir.Length + 1).Replace('\', '/')
    $hashLines += "$hash  $relPath"
}
$hashLines | Set-Content -Path (Join-Path $releaseDir "SHA256SUMS.txt") -Encoding UTF8

Write-Host "=== Release Build Complete ===" -ForegroundColor Green
Get-ChildItem -Path $releaseDir -Recurse -File | Select-Object FullName, Length
