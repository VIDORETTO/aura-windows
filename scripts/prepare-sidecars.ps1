# Builds aura-worker and places it where the Tauri bundler expects sidecars:
# apps/desktop/src-tauri/binaries/aura-worker-<target-triple>.exe
param([switch]$Release, [switch]$Engines, [switch]$DirectML, [switch]$Skip)

$ErrorActionPreference = "Stop"
$triple = (rustc -vV | Select-String "^host:").ToString().Split(" ")[1]
$profile = if ($Release) { "release" } else { "debug" }
$cargoArgs = @("build", "-p", "aura-worker")
if ($Release) { $cargoArgs += "--release" }
# Real ASR engines need CMake + MSVC (transcribe-rs); off by default in CI.
if ($DirectML) { $cargoArgs += @("--features", "directml") }
elseif ($Engines) { $cargoArgs += @("--features", "engines") }
# -Skip: keep the aura-worker already built in target/<profile> (only refresh the DLLs).
if (-not $Skip) {
  cargo @cargoArgs
  if ($LASTEXITCODE -ne 0) { throw "aura-worker build failed; refusing to bundle a stale sidecar" }
}
$dest = "apps/desktop/src-tauri/binaries"
New-Item -ItemType Directory -Force $dest | Out-Null
Copy-Item "target/$profile/aura-worker.exe" "$dest/aura-worker-$triple.exe" -Force
Write-Host "sidecar ready: $dest/aura-worker-$triple.exe"

# DLLs installed next to aura.exe/aura-worker.exe (bundle.resources "binaries/*.dll"):
# - the VC++ runtime: the worker's C++ engines link it dynamically and a clean
#   Windows has no Visual C++ Redistributable (app-local deployment of the Redist files);
# - DirectML.dll from the onnxruntime build (newer than Windows 10's System32 copy).
$vswhere = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio/Installer/vswhere.exe"
$vs = & $vswhere -latest -products * -property installationPath
$crt = Get-ChildItem (Join-Path $vs "VC/Redist/MSVC/*/x64/Microsoft.VC*.CRT") -Directory | Sort-Object FullName | Select-Object -Last 1
foreach ($dll in "vcruntime140.dll", "vcruntime140_1.dll", "msvcp140.dll", "msvcp140_1.dll") {
  Copy-Item (Join-Path $crt.FullName $dll) "$dest/$dll" -Force
}
if (Test-Path "target/$profile/DirectML.dll") { Copy-Item "target/$profile/DirectML.dll" "$dest/DirectML.dll" -Force }
Write-Host "runtime DLLs ready: $((Get-ChildItem $dest -Filter *.dll).Name -join ', ')"
