param(
    [Parameter(Mandatory)][string]$Zip,
    [Parameter(Mandatory)][string]$Evidence
)
$ErrorActionPreference = 'Stop'
$archive = (Resolve-Path -LiteralPath $Zip).Path
$hash = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
$expected = (Get-Content -LiteralPath "$archive.sha256" -Raw).Trim()
if ($expected -cne "$hash  $([IO.Path]::GetFileName($archive))") { throw 'ZIP checksum mismatch.' }
$temp = Join-Path ([IO.Path]::GetTempPath()) "amante-clean-$([Guid]::NewGuid().ToString('N'))"
New-Item -ItemType Directory -Path $temp | Out-Null
try {
    Expand-Archive -LiteralPath $archive -DestinationPath $temp
    $roots = @(Get-ChildItem -LiteralPath $temp -Directory)
    if ($roots.Count -ne 1) { throw 'Expected one portable installation folder.' }
    $root = $roots[0].FullName
    $exe = Join-Path $root 'amante-viewer.exe'
    if (!(Test-Path -LiteralPath $exe)) { throw 'Amante executable is missing.' }
    $identity = (Get-Item -LiteralPath $exe).VersionInfo
    if ($identity.ProductName -cne 'Amante Viewer' -or $identity.FileDescription -cne 'Amante Viewer' -or
        $identity.OriginalFilename -cne 'amante-viewer.exe') { throw 'Incorrect executable branding.' }
    $version = (& $exe --version | Out-String).Trim()
    if ($LASTEXITCODE -ne 0 -or $version -notmatch '^Amante Viewer \d+\.\d+\.\d+') { throw 'Extracted executable version failed.' }
    foreach ($name in @('LICENSE', 'NOTICE.md', 'README.md', 'assets/emoji/LICENSE-OFL.txt', 'aurora-install.json')) {
        if (!(Test-Path -LiteralPath (Join-Path $root $name))) { throw "Missing package file: $name" }
    }
    $metadata = Get-Content -LiteralPath (Join-Path $root 'aurora-install.json') -Raw | ConvertFrom-Json
    if ($metadata.schema -ne 1 -or "Amante Viewer $($metadata.version)" -cne $version) { throw 'Install metadata mismatch.' }
    $report = [ordered]@{
        schema = 1
        revision = (git rev-parse HEAD | Out-String).Trim()
        version = $version
        sha256 = $hash
        packageVerification = 'passed'
        extractedCli = 'passed: --version only; no GUI or login'
        desktopRuntime = 'not exercised; interactive Windows GPU desktop required'
    }
    $report | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $Evidence -Encoding utf8NoBOM
    $report | ConvertTo-Json
} finally { Remove-Item -LiteralPath $temp -Recurse -Force }
