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

if ($reason) {
    Step "Recompilando: $reason"
    # The running Aura keeps aura.exe locked; its children die with it (Job Object).
    $running = Get-Process aura -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq $exe }
    if ($running) {
        Step "Fechando o Aura em execução"
        $running | Stop-Process -Force
        Start-Sleep -Milliseconds 800
    }
    if (-not (Test-Path "apps\desktop\node_modules") -or
        (Get-Item "apps\desktop\pnpm-lock.yaml").LastWriteTime -gt (Get-Item "apps\desktop\node_modules").LastWriteTime) {
        Step "Instalando dependências da interface"
        pnpm -C apps/desktop install --frozen-lockfile
        if ($LASTEXITCODE -ne 0) { throw "pnpm install falhou" }
    }
    Step "Preparando o sidecar (aura-worker)"
    & (Join-Path $PSScriptRoot "prepare-sidecars.ps1")
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
    Step "Iniciando o Aura (Ctrl+Shift+Space abre e fecha o Overlay)"
    # Already running the same build: the single-instance plugin just shows it.
    Start-Process $exe
}
