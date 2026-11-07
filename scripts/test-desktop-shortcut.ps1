param([Parameter(Mandatory)][string]$Source)
$ErrorActionPreference = 'Stop'
$sourceRoot = (Resolve-Path -LiteralPath $Source).Path
$script = Join-Path $sourceRoot 'scripts/desktop-shortcut.ps1'
$temp = Join-Path ([IO.Path]::GetTempPath()) "amante-shortcut-$([Guid]::NewGuid().ToString('N'))"
New-Item -ItemType Directory -Path $temp | Out-Null
try {
    # Exercise the real COM shortcut API in a disposable folder, not the runner desktop.
    $exe = Join-Path $temp 'viewer with spaces.exe'
    Copy-Item -LiteralPath "$env:SystemRoot\System32\notepad.exe" -Destination $exe
    $shortcutPath = Join-Path $temp 'Amante Viewer Alpha.lnk'
    $shell = New-Object -ComObject WScript.Shell
    & powershell.exe -NoProfile -NonInteractive -File $script -Executable $exe -DesktopDirectory $temp
    if ($LASTEXITCODE -ne 0) { throw 'Shortcut creation failed.' }
    $link = $shell.CreateShortcut($shortcutPath)
    if ($link.TargetPath -ine $exe -or $link.WorkingDirectory -ine $temp) { throw 'Shortcut target mismatch.' }
    & powershell.exe -NoProfile -NonInteractive -File $script -Executable $exe -DesktopDirectory $temp
    if ($LASTEXITCODE -ne 0) { throw 'Repeated shortcut creation failed.' }
    & powershell.exe -NoProfile -NonInteractive -File $script -Executable $exe -DesktopDirectory $temp -Remove
    if ($LASTEXITCODE -ne 0 -or (Test-Path -LiteralPath $shortcutPath)) { throw 'Owned shortcut removal failed.' }
    $link = $shell.CreateShortcut($shortcutPath)
    $link.TargetPath = $exe
    $link.Description = 'User-owned shortcut'
    $link.Save()
    & powershell.exe -NoProfile -NonInteractive -File $script -Executable $exe -DesktopDirectory $temp
    if ($LASTEXITCODE -eq 0) { throw 'Expected an explicit refusal to replace a user-owned shortcut.' }
    & powershell.exe -NoProfile -NonInteractive -File $script -Executable $exe -DesktopDirectory $temp -Remove
    if ($LASTEXITCODE -ne 0 -or !(Test-Path -LiteralPath $shortcutPath)) { throw 'User-owned shortcut was changed.' }
    Write-Host 'Windows shortcut creation, repeat launch, removal, and user-owned shortcut protection passed.'
} finally {
    Remove-Item -LiteralPath $temp -Recurse -Force
}
