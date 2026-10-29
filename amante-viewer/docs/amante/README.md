# Amante Viewer

## Start on Windows

Extract the versioned Windows ZIP into its own folder and run
`amante-viewer.exe`. Do not run it from inside the ZIP or overwrite a running
installation. Keep your existing installation until you have checked the new
one. This is an unsigned development build.

Install the supported x64 Microsoft Visual C++ Redistributable from Microsoft's
official website if necessary, and use a Vulkan-capable GPU with its supported
driver. See [Windows build and prerequisites](WINDOWS_BUILD.md).

## Your interface

The login/landing screen, Help menu, About window, Preferences, creator tools,
notifications, resource monitor and English/French/Japanese application copy
use **Amante**. Your own content is not rebranded: residents' names, chat,
imported documents and existing grid inventory remain yours.

New viewer-owned inventory folders use `#Amante`, `Amante AO`,
`Amante Library` and `Amante Bridge`. Existing folders belonging to another
viewer are not automatically renamed, deleted or migrated.

## Offline first

To open the synthetic demo without grid credentials, use PowerShell:

```powershell
$env:AURORA_DEMO = '1'
.\amante-viewer.exe
Remove-Item Env:AURORA_DEMO
```

`AURORA_DEMO` is a retained compatibility switch, not the application's name.
No real password belongs in an automated script or a test report.

## Project links and services

The website/source buttons open
<https://github.com/tarahamante-cloud/Amante-viewer>. Help documentation lives
in that repository. Unconfigured community, privacy-policy, reporting,
production-update and hosted-plugin services must not redirect to an upstream
project under the Amante name.

- [Verified Windows builds](WINDOWS_VALIDATION.md)
- [Independent services](CONFIGURATION.md)
- [Security limitations](SECURITY.md)
- [Desktop checklist](DESKTOP_TESTING.md)

## License and authors

Amante is a fork of Aurora Viewer. Required upstream authorship and legal
notices remain intact in `LICENSE` and `NOTICE.md`. Those credits, historical
records and internal interoperability identifiers are intentionally retained;
they are not current product branding.
