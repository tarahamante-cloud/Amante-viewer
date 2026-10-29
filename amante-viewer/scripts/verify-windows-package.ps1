# Verify the actual extracted distribution, not just the build directory.
param(
    [Parameter(Mandatory)][string]$Zip,
    [Parameter(Mandatory)][string]$Evidence
)
$ErrorActionPreference = 'Stop'
$source = Split-Path $PSScriptRoot -Parent
$archive = (Resolve-Path -LiteralPath $Zip).Path
$hash = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
$checksum = (Get-Content -LiteralPath "$archive.sha256" -Raw).Trim()
if ($checksum -cne "$hash  $([IO.Path]::GetFileName($archive))") { throw 'ZIP checksum sidecar mismatch.' }
$temp = Join-Path ([IO.Path]::GetTempPath()) "amante-package-check-$([Guid]::NewGuid().ToString('N'))"
New-Item -ItemType Directory -Path $temp | Out-Null
try {
    Expand-Archive -LiteralPath $archive -DestinationPath $temp
    $roots = @(Get-ChildItem -LiteralPath $temp -Directory)
    if ($roots.Count -ne 1) { throw 'Expected one portable installation folder.' }
    $root = $roots[0].FullName
    $exe = Join-Path $root 'amante-viewer.exe'
    if (!(Test-Path -LiteralPath $exe -PathType Leaf)) { throw 'Amante executable missing.' }
    if (Test-Path -LiteralPath (Join-Path $root 'aurora-viewer.exe')) { throw 'Unexpected upstream executable.' }
    $identity = (Get-Item -LiteralPath $exe).VersionInfo
    foreach ($field in @('ProductName', 'FileDescription')) {
        if ($identity.$field -cne 'Amante Viewer') { throw "Incorrect Windows $field." }
    }
    if ($identity.OriginalFilename -cne 'amante-viewer.exe') { throw 'Incorrect original filename resource.' }
    $reader = [IO.BinaryReader]::new([IO.File]::OpenRead($exe))
    try {
        if ($reader.ReadUInt16() -ne 0x5A4D) { throw 'Missing DOS executable header.' }
        $reader.BaseStream.Position = 0x3C
        $peOffset = $reader.ReadUInt32()
        $reader.BaseStream.Position = $peOffset
        if ($reader.ReadUInt32() -ne 0x4550 -or $reader.ReadUInt16() -ne 0x8664) { throw 'Not an x64 PE executable.' }
    } finally { $reader.Dispose() }
    # Executing the extracted binary catches missing DLLs and wrong CLI identity.
    Push-Location $root
    try {
        $version = (& $exe --version | Out-String).Trim()
        if ($LASTEXITCODE -ne 0 -or $version -notmatch '^Amante Viewer \d+\.\d+\.\d+([+-][0-9A-Za-z.+-]+)?$') {
            throw 'Extracted executable version check failed.'
        }
    } finally { Pop-Location }
    $files = @('LICENSE', 'NOTICE.md', 'README.md', 'AMANTE.md', 'assets/emoji/LICENSE-OFL.txt')
    foreach ($relative in $files) {
        $sourceRelative = if ($relative -eq 'README.md') { 'docs/amante/README.md' } else { $relative }
        $original = Join-Path $source $sourceRelative
        $packaged = Join-Path $root $relative
        if (!(Test-Path -LiteralPath $packaged -PathType Leaf) -or
            (Get-FileHash -LiteralPath $original).Hash -cne (Get-FileHash -LiteralPath $packaged).Hash) {
            throw "Missing or changed package file: $relative"
        }
    }
    $dictionaries = Join-Path $source 'crates/aurora-viewer/assets/dictionaries'
    foreach ($dictionary in Get-ChildItem -LiteralPath $dictionaries -Directory) {
        $original = Join-Path $dictionary.FullName 'LICENSE.txt'
        if (!(Test-Path -LiteralPath $original)) { continue }
        $packaged = Join-Path $root "assets/dictionaries/$($dictionary.Name)-LICENSE.txt"
        if (!(Test-Path -LiteralPath $packaged) -or
            (Get-FileHash -LiteralPath $original).Hash -cne (Get-FileHash -LiteralPath $packaged).Hash) {
            throw "Missing or changed dictionary license: $($dictionary.Name)"
        }
    }
    if (!(Test-Path -LiteralPath (Join-Path $root 'assets/dictionaries/README.md'))) { throw 'Dictionary source list missing.' }
    foreach ($doc in Get-ChildItem -LiteralPath (Join-Path $source 'docs/amante') -File) {
        $packaged = Join-Path $root "docs/amante/$($doc.Name)"
        if (!(Test-Path -LiteralPath $packaged) -or
            (Get-FileHash -LiteralPath $doc.FullName).Hash -cne (Get-FileHash -LiteralPath $packaged).Hash) {
            throw "Missing or changed Amante documentation: $($doc.Name)"
        }
    }
    $metadata = Get-Content -LiteralPath (Join-Path $root 'aurora-install.json') -Raw | ConvertFrom-Json
    if ($metadata.schema -ne 1 -or "Amante Viewer $($metadata.version)" -cne $version) { throw 'Install metadata/version mismatch.' }
    $revision = (git -C $source rev-parse HEAD | Out-String).Trim()
    if ($LASTEXITCODE -ne 0) { throw 'Cannot identify source revision.' }
    $report = [ordered]@{
        schema = 1
        revision = $revision
        runUrl = "https://github.com/$env:GITHUB_REPOSITORY/actions/runs/$env:GITHUB_RUN_ID"
        os = [Environment]::OSVersion.VersionString
        zip = [IO.Path]::GetFileName($archive)
        sha256 = $hash
        version = $version
        nativeBuild = 'passed'
        packageVerification = 'passed'
        windowsResources = 'passed: product, description, original filename, x64 PE'
        extractedCli = 'passed: --version only; no GUI or login'
        assetsAndLicenses = 'passed: packaged license/source/doc files match; renderer assets embedded at build time'
        desktopRuntime = 'not exercised; interactive Windows GPU desktop required'
        profileRuntime = 'not exercised by package verification; see offline harness and human checklist'
    }
    $report | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $Evidence -Encoding utf8NoBOM
    $report | ConvertTo-Json -Depth 5
} finally { Remove-Item -LiteralPath $temp -Recurse -Force }
