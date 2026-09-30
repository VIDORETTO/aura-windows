# Builds aura-worker and places it where the Tauri bundler expects sidecars:
# apps/desktop/src-tauri/binaries/aura-worker-<target-triple>.exe
param([switch]$Release, [switch]$Engines)

$ErrorActionPreference = "Stop"
$triple = (rustc -vV | Select-String "^host:").ToString().Split(" ")[1]
$profile = if ($Release) { "release" } else { "debug" }
$cargoArgs = @("build", "-p", "aura-worker")
if ($Release) { $cargoArgs += "--release" }
# Real ASR engines need CMake + MSVC (transcribe-rs); off by default in CI.
if ($Engines) { $cargoArgs += @("--features", "engines") }
cargo @cargoArgs
$dest = "apps/desktop/src-tauri/binaries"
New-Item -ItemType Directory -Force $dest | Out-Null
Copy-Item "target/$profile/aura-worker.exe" "$dest/aura-worker-$triple.exe" -Force
Write-Host "sidecar ready: $dest/aura-worker-$triple.exe"
