param(
    [Parameter(Mandatory)][string]$Executable,
    [string]$DesktopDirectory = [Environment]::GetFolderPath('Desktop'),
    [switch]$Remove
)
$ErrorActionPreference = 'Stop'
if (!$DesktopDirectory) { throw 'Windows desktop folder is unavailable.' }
$path = Join-Path $DesktopDirectory 'Amante Viewer Alpha.lnk'
$shell = New-Object -ComObject WScript.Shell
$exists = Test-Path -LiteralPath $path
$shortcut = $shell.CreateShortcut($path)
$owner = 'Amante Viewer Alpha desktop shortcut'
if ($Remove) {
    if ($exists -and $shortcut.Description -ceq $owner -and
        $shortcut.TargetPath -ieq $Executable) {
        Remove-Item -LiteralPath $path
    }
    exit 0
}
if (!(Test-Path -LiteralPath $Executable -PathType Leaf)) { throw 'Viewer executable is missing.' }
if ($exists -and $shortcut.Description -cne $owner) {
    throw 'A different shortcut already uses this name; it was not changed.'
}
$shortcut.TargetPath = $Executable
$shortcut.WorkingDirectory = Split-Path -Parent $Executable
$shortcut.IconLocation = "$Executable,0"
$shortcut.Description = $owner
$shortcut.Save()
