param(
    [Parameter(Mandatory)] [string] $CodexBin,
    [Parameter(Mandatory)] [string] $Output
)
$ErrorActionPreference = 'Stop'
$projectRoot = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$outputPath = [IO.Path]::GetFullPath($Output)
if (Test-Path -LiteralPath $outputPath) { throw 'A new result path is required' }
$fixtureRoot = Join-Path ([IO.Path]::GetTempPath()) ('aura-039-encrypted-' + [guid]::NewGuid())
[IO.Directory]::CreateDirectory($fixtureRoot) | Out-Null
$receiptPath = Join-Path $fixtureRoot 'provider-received.txt'
$logPath = Join-Path $fixtureRoot 'native-test.log'
$diagnosticPath = Join-Path $fixtureRoot 'diagnostics'
[IO.Directory]::CreateDirectory($diagnosticPath) | Out-Null
# Only public synthetic bytes. The key stays in this independent .NET probe;
# neither the Aura runtime nor the controlled provider receives it.
$plain = [Text.Encoding]::UTF8.GetBytes('Independent summary https://independent.example/report Production: 30 units.')
$key = [Security.Cryptography.RandomNumberGenerator]::GetBytes(32)
$nonce = [Security.Cryptography.RandomNumberGenerator]::GetBytes(12)
$cipher = [byte[]]::new($plain.Length)
$tag = [byte[]]::new(16)
$aes = [Security.Cryptography.AesGcm]::new($key, 16)
$prior = @{}
$names = @('AURA_CODEX_BIN','AURA_E2E_DIR','AURA_WEB_REASONING_CIPHERTEXT','AURA_WEB_REASONING_RECEIPT','AURA_WEB_KEEP_SYNTHETIC_PROFILE','AURA_GATEWAY_DUMP_DIR')
foreach ($name in $names) { $prior[$name] = [Environment]::GetEnvironmentVariable($name, 'Process') }
try {
    $aes.Encrypt($nonce, $plain, $cipher, $tag)
    $env:AURA_CODEX_BIN = (Resolve-Path -LiteralPath $CodexBin).Path
    $env:AURA_E2E_DIR = $fixtureRoot
    $env:AURA_WEB_REASONING_CIPHERTEXT = [Convert]::ToBase64String($cipher + $tag)
    $env:AURA_WEB_REASONING_RECEIPT = $receiptPath
    $env:AURA_WEB_KEEP_SYNTHETIC_PROFILE = '1'
    $env:AURA_GATEWAY_DUMP_DIR = $diagnosticPath
    Push-Location -LiteralPath $projectRoot
    try {
        cargo test -p aura-app --test real_app_server web_compaction_preserves_history_without_reintroducing_private_reasoning -- --ignored --nocapture --test-threads=1 2>&1 | Tee-Object -FilePath $logPath
        if ($LASTEXITCODE -ne 0) { throw 'Native encrypted reasoning journey failed' }
        if ((Get-ChildItem -LiteralPath $diagnosticPath -File).Count -ne 0) { throw 'Native private requests produced raw diagnostics' }
        cargo test -p aura-gateway --test private_calls private_web_diagnostics_are_suppressed -- --ignored --nocapture --test-threads=1 2>&1 | Tee-Object -Append -FilePath $logPath
        if ($LASTEXITCODE -ne 0) { throw 'Diagnostic positive control or private suppression failed' }
    } finally { Pop-Location }
    $received = [IO.File]::ReadAllText($receiptPath)
    if ($received -cne $env:AURA_WEB_REASONING_CIPHERTEXT) { throw 'Provider did not receive the original ciphertext' }
    $packed = [Convert]::FromBase64String($received)
    $receivedCipher = [byte[]]$packed[0..($packed.Length - 17)]
    $receivedTag = [byte[]]$packed[($packed.Length - 16)..($packed.Length - 1)]
    $decoded = [byte[]]::new($receivedCipher.Length)
    $aes.Decrypt($nonce, $receivedCipher, $receivedTag, $decoded)
    if ([Text.Encoding]::UTF8.GetString($decoded) -cne 'Independent summary https://independent.example/report Production: 30 units.') {
        throw 'Independent decryption did not recover the literal synthetic payload'
    }
    # Verify that this oracle actually checks authentication, not just decoding.
    $receivedTag[0] = $receivedTag[0] -bxor 1
    $tamperRejected = $false
    try { $aes.Decrypt($nonce, $receivedCipher, $receivedTag, $decoded) }
    catch [Security.Cryptography.CryptographicException] { $tamperRejected = $true }
    if (-not $tamperRejected) { throw 'Authentication control accepted a tampered tag' }
    $profileLine = Select-String -LiteralPath $logPath -Pattern 'Synthetic privacy fixture retained at (.+)' | Select-Object -Last 1
    if (-not $profileLine) { throw 'Own native profile was not retained for SQLite inspection' }
    $profilePath = [IO.Path]::GetFullPath($profileLine.Matches[0].Groups[1].Value.Trim())
    if (-not $profilePath.StartsWith($fixtureRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Native profile is outside this synthetic probe directory'
    }
    $auditOutput = Join-Path $fixtureRoot 'storage-audit.json'
    python (Join-Path $PSScriptRoot 'audit_synthetic_storage.py') (Join-Path $profilePath 'Aura') $auditOutput --reasoning --compaction
    if ($LASTEXITCODE -ne 0) { throw 'Independent storage audit failed' }
    $audit = Get-Content -LiteralPath $auditOutput -Raw | ConvertFrom-Json
    if ($audit.hits.Count -ne 0) { throw 'Independent auditor found a private synthetic marker' }
    [ordered]@{
        nativeTestPassed = $true
        algorithm = 'AES-256-GCM; external synthetic provider fixture'
        ciphertextBytesPreservedAtProvider = $true
        independentAuthenticatedDecryption = $true
        tamperedTagRejected = $true
        nativeRestartAndCompaction = $true
        nativePrivateLiteralAndCiphertextScan = 'passed'
        nativePrivateDiagnosticCount = 0
        gatewayPublicDiagnosticPositiveControl = $true
        gatewayPrivateDiagnosticCount = 0
        gatewayPrivateProviderErrorSanitized = $true
        independentStorageAudit = [ordered]@{files=$audit.files;sqliteRows=$audit.sqliteRows;compressedValues=$audit.compressedValues;hits=$audit.hits.Count}
        limitations = @('Representative encrypted field only; no commercial model execution',
                        'Aura treats ciphertext as opaque; no vendor encryption format is claimed',
                        'Native scan does not establish general absence of all derived information')
    } | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $outputPath -Encoding utf8
    Write-Output ('Sanitized result: ' + $outputPath)
} finally {
    foreach ($name in $names) { [Environment]::SetEnvironmentVariable($name, $prior[$name], 'Process') }
    $aes.Dispose()
    [Array]::Clear($key, 0, $key.Length)
    Write-Output ('Synthetic probe files retained at ' + $fixtureRoot)
}
