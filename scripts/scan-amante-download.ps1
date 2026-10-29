# Scan an existing CI download without executing or repackaging its contents.
# Run only on Windows with Microsoft Defender available.
#requires -Version 7.0
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$InputDirectory,
    [Parameter(Mandatory = $true)]
    [string]$ReportDirectory
)

$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
New-Item -ItemType Directory -Force -Path $ReportDirectory | Out-Null
$report = [ordered]@{
    status = 'not_completed'
    source_run_id = $env:SOURCE_RUN_ID
    started_at_utc = [DateTime]::UtcNow.ToString('o')
    package = $null
    sha256 = $null
    checksum_matches_sidecar = $false
    scanner = $null
    scanner_exit_code = $null
    package_executed = $false
    google_safe_browsing_checked = $false
    limitation = 'A matching unsigned checksum is not proof of authenticity. A clean Defender scan does not prove safety or clear a Chrome warning.'
}

try {
    if (-not $IsWindows) { throw 'This diagnostic requires Windows and Microsoft Defender.' }
    $packages = @(Get-ChildItem -LiteralPath $InputDirectory -File -Filter 'Amante-Viewer-v*-windows-x64.zip')
    if ($packages.Count -ne 1) { throw 'Expected exactly one original Amante Windows package ZIP.' }
    $package = $packages[0]
    $report.package = $package.Name
    $report.sha256 = (Get-FileHash -LiteralPath $package.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    $sidecar = Get-Content -LiteralPath "$($package.FullName).sha256" -Raw
    if ($sidecar -notmatch '^([a-fA-F0-9]{64})[ \t]+\*?([^\r\n]+)\r?\n?$') {
        throw 'Missing or malformed package checksum.'
    }
    if ($Matches[1].ToLowerInvariant() -ne $report.sha256 -or $Matches[2] -cne $package.Name) {
        throw 'The package does not match its checksum sidecar.'
    }
    $report.checksum_matches_sidecar = $true

    $platform = Join-Path $env:ProgramData 'Microsoft\Windows Defender\Platform'
    $candidates = @(Get-ChildItem -Path "$platform\*\MpCmdRun.exe" -File -ErrorAction SilentlyContinue |
        Sort-Object LastWriteTime -Descending)
    $scanner = if ($candidates.Count -gt 0) {
        $candidates[0].FullName
    } else {
        Join-Path $env:ProgramFiles 'Windows Defender\MpCmdRun.exe'
    }
    if (-not (Test-Path -LiteralPath $scanner -PathType Leaf)) {
        throw 'Microsoft Defender scanner is unavailable; no clean result can be claimed.'
    }
    Update-MpSignature -ErrorAction Stop
    $status = Get-MpComputerStatus -ErrorAction Stop
    $report.scanner = [ordered]@{
        name = 'Microsoft Defender'
        product_version = $status.AMProductVersion
        engine_version = $status.AMEngineVersion
        signature_version = $status.AntivirusSignatureVersion
        signature_updated = $status.AntivirusSignatureLastUpdated
        service_enabled = $status.AMServiceEnabled
        antivirus_enabled = $status.AntivirusEnabled
    }
    if (-not $status.AMServiceEnabled -or -not $status.AntivirusEnabled) {
        throw 'Defender is not available for this scan; no clean result can be claimed.'
    }

    # Microsoft's custom-scan option scans archives and ignores file exclusions,
    # but applies no remediation. It does not disable antivirus protection.
    # Detections are in command output, not the Defender UI/event log.
    $scanOutput = & $scanner -Scan -ScanType 3 -File $package.FullName -DisableRemediation 2>&1
    $scanExitCode = $LASTEXITCODE
    $scanOutput | Out-File -LiteralPath (Join-Path $ReportDirectory 'defender-scan.txt') -Encoding utf8
    $scanOutput | ForEach-Object { Write-Output "$_" }
    $report.scanner_exit_code = $scanExitCode
    if ($scanExitCode -ne 0) {
        $report.status = 'detection_or_scan_error'
        throw "Defender returned $scanExitCode. Inspect defender-scan.txt; do not treat this as a clean scan."
    }
    $afterScan = (Get-FileHash -LiteralPath $package.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($afterScan -ne $report.sha256) { throw 'The package changed during scanning.' }
    $report.status = 'no_defender_detection'
    Write-Output 'No Microsoft Defender detection reported. Google Safe Browsing remains unverified.'
} catch {
    $report['error'] = $_.Exception.Message
    throw
} finally {
    $report | ConvertTo-Json -Depth 5 |
        Out-File -LiteralPath (Join-Path $ReportDirectory 'scan-report.json') -Encoding utf8
}
