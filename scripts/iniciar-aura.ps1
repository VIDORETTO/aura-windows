# Starts the newest Aura build: recompiles only when the sources are newer than
# target\release\aura.exe (or the build flavor changed), stopping the running
# copy first because Windows keeps the .exe locked while it runs.
#
#   .\iniciar-aura.bat                 # real mode
#   .\iniciar-aura.bat -Demo           # fake agent + synthetic OS (no account)
#   .\iniciar-aura.bat -Rebuild        # force a full rebuild
#   .\iniciar-aura.bat -Pull           # git pull --ff-only before building
#   .\iniciar-aura.bat -NoStart        # just build
param([switch]$Demo, [switch]$Rebuild, [switch]$Pull, [switch]$NoStart)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

$exe = Join-Path $root "target\release\aura.exe"
$worker = Join-Path $root "target\release\aura-worker.exe"
$stamp = Join-Path $root "target\release\.aura-build"
$flavor = if ($Demo) { "demo" } else { "real" }

function Step($text) { Write-Host "» $text" -ForegroundColor Cyan }

if ($Pull) {
    Step "Atualizando o repositório (git pull --ff-only)"
    git pull --ff-only
    if ($LASTEXITCODE -ne 0) { throw "git pull falhou" }
}

# Newest source file vs the executable.
$sources = @(
    "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", ".cargo",
    "crates", "tools",
    "apps\desktop\src", "apps\desktop\public", "apps\desktop\index.html",
    "apps\desktop\package.json", "apps\desktop\pnpm-lock.yaml", "apps\desktop\vite.config.ts",
    "apps\desktop\src-tauri\src", "apps\desktop\src-tauri\capabilities", "apps\desktop\src-tauri\icons",
    "apps\desktop\src-tauri\Cargo.toml", "apps\desktop\src-tauri\tauri.conf.json", "apps\desktop\src-tauri\build.rs"
) | ForEach-Object { Join-Path $root $_ } | Where-Object { Test-Path $_ }
# Files via Get-Item: `Get-ChildItem <file> -Recurse` treats the name as a
# filter and walks the whole repo (target\, node_modules\) looking for it.
$files = $sources | Where-Object { -not (Test-Path $_ -PathType Container) } | ForEach-Object { Get-Item $_ }
$dirs = $sources | Where-Object { Test-Path $_ -PathType Container }
$newest = @($files) + @(Get-ChildItem $dirs -Recurse -File -ErrorAction SilentlyContinue |
    Where-Object { $_.FullName -notmatch '\\(node_modules|target|dist|gen)\\' }) |
    Sort-Object LastWriteTime -Descending | Select-Object -First 1

$reason = $null
if ($Rebuild) { $reason = "recompilação pedida" }
elseif (-not (Test-Path $exe)) { $reason = "ainda não há build" }
elseif (-not (Test-Path $stamp) -or (Get-Content $stamp -Raw).Trim() -ne $flavor) { $reason = "o build atual não é do modo '$flavor'" }
elseif ($newest -and $newest.LastWriteTime -gt (Get-Item $exe).LastWriteTime) {
    $reason = "código mais novo que o executável ($($newest.FullName.Substring($root.Length + 1)))"
}

# The speech worker (local voice) is a separate executable next to aura.exe:
# rebuild it when it is missing, older than its sources or without engines.
$workerReason = $null
if (-not $Demo) {
    $workerNewest = Get-ChildItem (Join-Path $root "crates\aura-worker"), (Join-Path $root "crates\aura-asr") -Recurse -File |
        Where-Object { $_.FullName -notmatch '\\target\\' } | Sort-Object LastWriteTime -Descending | Select-Object -First 1
    if ($Rebuild -or $reason) { $workerReason = "acompanha o app" }
    elseif (-not (Test-Path $worker)) { $workerReason = "worker de voz ausente" }
    elseif ($workerNewest.LastWriteTime -gt (Get-Item $worker).LastWriteTime) { $workerReason = "código do worker mais novo" }
    else {
        $caps = & $worker --capabilities 2>$null
        if (-not ("$caps" -match "onnx-parakeet")) { $workerReason = "worker sem motores de voz" }
    }
}

function Stop-RunningAura {
    # The running Aura keeps aura.exe locked; its children die with it (Job Object).
    $running = Get-Process aura -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq $exe }
    if ($running) {
        Step "Fechando o Aura em execução"
        $running | Stop-Process -Force
        Start-Sleep -Milliseconds 800
    }
}

if ($workerReason) {
    Step "Preparando o worker de voz com os motores locais ($workerReason)"
    Stop-RunningAura
    # transcribe-rs needs libclang (bindgen) and CMake; use the copy in target\qa-tools when present.
    $clang = Join-Path $root "target\qa-tools\libclang18\libclang-18.1.1.data\platlib\clang\native"
    if (-not $env:LIBCLANG_PATH -and (Test-Path (Join-Path $clang "libclang.dll"))) { $env:LIBCLANG_PATH = $clang }
    & (Join-Path $PSScriptRoot "prepare-sidecars.ps1") -Release -Engines
    if ($LASTEXITCODE -ne 0) { throw "o worker de voz não compilou (precisa de CMake e libclang; defina LIBCLANG_PATH)" }
    Copy-Item (Join-Path $root "target\release\aura-worker.exe") $worker -Force -ErrorAction SilentlyContinue
}

if ($reason) {
    Step "Recompilando: $reason"
    Stop-RunningAura
    if (-not (Test-Path "apps\desktop\node_modules") -or
        (Get-Item "apps\desktop\pnpm-lock.yaml").LastWriteTime -gt (Get-Item "apps\desktop\node_modules").LastWriteTime) {
        Step "Instalando dependências da interface"
        pnpm -C apps/desktop install --frozen-lockfile
        if ($LASTEXITCODE -ne 0) { throw "pnpm install falhou" }
    }
    if ($Demo) {
        Step "Preparando o sidecar (aura-worker)"
        & (Join-Path $PSScriptRoot "prepare-sidecars.ps1")
    }
    $buildArgs = @("-C", "apps/desktop", "tauri", "build", "--no-bundle")
    if ($Demo) { $buildArgs += @("--features", "demo") }
    Step "Compilando o app (pnpm tauri build --no-bundle$(if ($Demo) { ' --features demo' }))"
    pnpm @buildArgs
    if ($LASTEXITCODE -ne 0) { throw "a compilação falhou (veja a saída acima)" }
    Set-Content -Path $stamp -Value $flavor -NoNewline
} else {
    Step "O executável já é a versão mais recente ($flavor)"
}

if (-not $NoStart) {
    # The first start goes to the tray (001 AC-001); starting it again opens the Overlay.
    Step "Iniciando o Aura na bandeja (Ctrl+Shift+Space ou executar de novo abre o Overlay)"
    # Already running the same build: the single-instance plugin just shows it.
    Start-Process $exe
}
