# Build every workspace unit/integration test, then execute only those binaries
# with outbound non-loopback traffic blocked. No grid credentials are supplied.
param([Parameter(Mandatory)][string]$Out)
$ErrorActionPreference = 'Stop'
New-Item -ItemType Directory -Path $Out -Force | Out-Null
$Out = (Resolve-Path -LiteralPath $Out).Path
$memory = (Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory
$revision = (git rev-parse HEAD | Out-String).Trim()
if ($LASTEXITCODE -ne 0) { throw 'Cannot identify source revision.' }
$summary = [ordered]@{ schema = 1; revision = $revision; memoryBytes = $memory; compile = 'not completed'; tests = @() }
$summaryPath = Join-Path $Out 'summary.json'
$summary | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $summaryPath -Encoding utf8NoBOM
Write-Host "Physical memory: $memory bytes"
$buildLog = Join-Path $Out 'compile.jsonl'
cargo test --profile ci --workspace --locked --no-run --message-format=json 1> $buildLog
if ($LASTEXITCODE -ne 0) { throw 'Full native test harness did not compile. See compile.jsonl and Actions stderr.' }
$summary.compile = 'passed'
$binaries = @(
    Get-Content -LiteralPath $buildLog | ForEach-Object {
        $message = $_ | ConvertFrom-Json
        if ($message.reason -eq 'compiler-artifact' -and $message.profile.test -and $message.executable) {
            $message.executable
        }
    } | Sort-Object -Unique
)
if ($binaries.Count -eq 0 -or !($binaries | Where-Object { [IO.Path]::GetFileName($_) -like 'aurora_viewer-*' })) {
    throw 'Complete viewer test executable was not emitted.'
}
$group = "Amante offline tests $([Guid]::NewGuid().ToString('N'))"
try {
    # Do not depend on Cargo --offline: that flag constrains dependency fetching,
    # not the networking performed by a running test. Keep IPv4 127/8 and ::1.
    $ranges = @('0.0.0.0-126.255.255.255', '128.0.0.0-255.255.255.255', '::2-ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff')
    foreach ($binary in $binaries) {
        New-NetFirewallRule -DisplayName "Amante offline $([IO.Path]::GetFileName($binary))" -Group $group `
            -Direction Outbound -Program $binary -Action Block -RemoteAddress $ranges -Profile Any | Out-Null
    }
    $rules = @(Get-NetFirewallRule -Group $group)
    if ($rules.Count -ne $binaries.Count -or ($rules | Where-Object { $_.Enabled -ne 'True' })) {
        throw 'Offline test firewall rules are not active.'
    }
    if (Get-NetFirewallProfile | Where-Object { !$_.Enabled }) { throw 'Windows firewall profiles must be enabled.' }
    $failed = $false
    foreach ($binary in $binaries) {
        $name = [IO.Path]::GetFileNameWithoutExtension($binary)
        $log = Join-Path $Out "$name.txt"
        & $binary --test-threads=2 2>&1 | Tee-Object -FilePath $log
        $code = $LASTEXITCODE
        $summary.tests += [ordered]@{ binary = [IO.Path]::GetFileName($binary); exitCode = $code; status = $(if ($code -eq 0) { 'passed' } else { 'failed' }); log = "$name.txt" }
        $summary | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $summaryPath -Encoding utf8NoBOM
        if ($code -ne 0) { $failed = $true }
    }
    if ($failed) { throw 'One or more offline native suites failed. See summary.json.' }
} finally {
    Get-NetFirewallRule -Group $group -ErrorAction SilentlyContinue | Remove-NetFirewallRule
    $summary | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $summaryPath -Encoding utf8NoBOM
}
