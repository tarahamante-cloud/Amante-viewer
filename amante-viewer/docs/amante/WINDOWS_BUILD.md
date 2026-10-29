# Windows x64 build and download

## GitHub Actions

Keep the repository-root layout: `amante-viewer/`, root `.github/workflows/`,
and the source-inventory scripts under root `scripts/`. Unrelated Replit starter
apps, local data and build outputs are not required. Run
**Amante native checks and Windows download**
from its Actions page. Download `Amante-Viewer-windows-x64` from the successful
run's artifacts, then extract the versioned ZIP inside it.

The workflow:

- checks out with commit-SHA-pinned official actions and no persisted credential;
- uses read-only repository permissions and no repository/signing/OAuth secrets;
- compiles the real Rust/native viewer with the locked dependency graph;
- packages `amante-viewer.exe`, retained required assets and third-party licenses;
- writes a SHA-256 checksum next to the ZIP;
- extracts the ZIP into a fresh temporary directory, checks the x64 PE and
  Windows product resources, runs the extracted `--version`, and compares
  packaged licenses and documentation with their source files;
- runs the complete workspace unit/integration executables in a separate
  Windows job, with outbound non-loopback test traffic blocked;
- never signs in, contacts a live grid, publishes a release or enables updates.

The Windows test job records actual runner memory, compilation output, each
test binary's exit code and test log in `Amante-Windows-offline-test-evidence`.
Ignored tests remain ignored: graphics, audio devices, real credential-store
round trips and other explicitly manual fixtures are not silently certified.
The ZIP artifact also includes its package-verification report, source blob
comparison and a per-declaration feature CSV. A whole-target build/package
result is not evidence that any particular UI action was exercised.

Windows uses `windows-2025` and Rust 1.99.1. Linux checks retain the native
Linux prerequisites. Original platform/configuration files remain in the tree.
The original workflows inside `amante-viewer/.github` are nested upstream
reference files, **not this workspace's active Actions workflows**.
Do not export them as active top-level release automation without auditing and
replacing upstream ownership, endpoints, permissions and action references.

See [Windows validation record](WINDOWS_VALIDATION.md) for actual runs and
evidence. This Linux workspace cannot itself validate interactive Windows
rendering, devices, native dialogs, or the graphical updater fixture.

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

## Windows runtime prerequisites

The development executable is not a self-contained Windows installation. It
imports `MSVCP140.dll`, `VCRUNTIME140.dll` and `VCRUNTIME140_1.dll`; the ZIP does
not redistribute Microsoft's runtime. Install the supported **x64 Microsoft
Visual C++ Redistributable** from
[Microsoft's official instructions](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist)
if it is not already installed, following your organization's security policy.
Do not download individual DLLs from third-party sites. Installing prerequisites
may require administrator approval even though the portable viewer itself does
not.

A working Vulkan-capable GPU and its vendor's supported graphics driver are
also required. CI's successful extracted `--version` proves loading on the
hosted runner, which already has developer runtimes; it does not certify a clean
Windows machine or the native graphics path. This is an unsigned development
build, not a signed release or a reason to bypass Windows security warnings.

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
