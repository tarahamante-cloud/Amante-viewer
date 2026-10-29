# Windows x64 build and download

## GitHub Actions

Push the **whole workspace root**, including root `.github/workflows`, to an
owner-controlled repository. Run **Amante native checks and Windows download**
from its Actions page. Download `Amante-Viewer-windows-x64` from the successful
run's artifacts, then extract the versioned ZIP inside it.

The workflow:

- checks out with commit-SHA-pinned official actions and no persisted credential;
- uses read-only repository permissions and no repository/signing/OAuth secrets;
- compiles the real Rust/native viewer with the locked dependency graph;
- packages `amante-viewer.exe`, retained required assets and third-party licenses;
- writes a SHA-256 checksum next to the ZIP;
- never signs in, contacts a live grid, publishes a release or enables updates.

Windows uses `windows-2025` and Rust 1.99.1. Linux checks retain the native
Linux prerequisites. Original platform/configuration files remain in the tree.
The original workflows inside `amante-viewer/.github` are nested upstream
reference files, **not this workspace's active Actions workflows**.
Do not export them as active top-level release automation without auditing and
replacing upstream ownership, endpoints, permissions and action references.

**No Windows run/download is claimed to exist yet.** This Linux workspace has
not validated the MSVC/Windows SDK build or exercised the ZIP.

## Local Windows build

Install Git, Rust 1.99.1 x64 MSVC, Visual Studio C++ Build Tools and a Windows SDK,
plus the native tools documented in upstream `README.md`. In PowerShell:

```powershell
cd amante-viewer
cargo build --release --locked -p aurora-viewer
if ($LASTEXITCODE -ne 0) { throw 'Build failed' }
./scripts/package.ps1 -Exe target/release/aurora-viewer.exe -Out ../generated-artifacts/Amante-Viewer
../generated-artifacts/Amante-Viewer/amante-viewer.exe --version
```

The internal Cargo target still has its upstream name; the packaging step
creates the Amante executable. Launch the packaged file, not an Aurora install.
No administrator rights should be required for the portable package itself.

Run the first graphics test in offline mode:

```powershell
$env:AURORA_DEMO = '1'
../generated-artifacts/Amante-Viewer/amante-viewer.exe --title 'Amante offline validation'
Remove-Item Env:AURORA_DEMO
```

Keep the asset/license folders with the executable. Record the ZIP SHA-256,
build revision, OS, GPU and driver when reporting test results. An unsigned
development package may trigger Windows warnings; do not bypass organizational
security policy or describe it as signed.
