# Amante Viewer

This is a native desktop fork of Aurora Viewer, not a web replacement.
All upstream crates, native components, assets, tools, and platform files are
retained. Internal `aurora-*` crate names and compatibility identifiers are not
renamed merely for appearance.

## Immutable upstream baseline

- Repository: https://github.com/Aurora-Viewer/Aurora-Viewer
- Branch: `main`
- Commit: `d555fb677004bee8fd9e7f12b0e33a8978e77d4c`
- Commit subject: `feat(inventory): default hover actions to Add (#1377)`
- Declared submodules: none.
- Git LFS tracked files: none.
- License: GPL-3.0-or-later; upstream `LICENSE` and `NOTICE.md` are retained.

The source inventory is captured from this commit's immutable Git objects in
`docs/amante/upstream-inventory.json` before application edits. It includes
every tracked file and blob identity, UI source modules, menu call sites,
settings declarations, keybindings/toolbar declarations, and locale keys.
Its source-preservation status is **not** a claim that the features have been
run or tested.

## Delivery status

Development is in progress. Do not distribute this checkout as a completed,
security-validated Amante release. Build results, outstanding configuration,
security findings, and desktop test gaps must be reported explicitly.

- [Feature/source parity evidence](docs/amante/FEATURE_PARITY.md)
- [Security fixes and unresolved findings](docs/amante/SECURITY.md)
- [Build evidence and limitations](docs/amante/BUILD_STATUS.md)
- [Windows build and download instructions](docs/amante/WINDOWS_BUILD.md)
- [Independent service configuration](docs/amante/CONFIGURATION.md)
- [Service ownership and multilingual content audit](docs/amante/SERVICE_OWNERSHIP.md)
- [Human desktop test checklist](docs/amante/DESKTOP_TESTING.md)

The distributed executable is `amante-viewer.exe` on Windows and
`amante-viewer` on Linux. Cargo's internal `aurora-viewer` target is retained;
the packaging script gives it the Amante distribution name.

Replit imports the source into its project repository rather than preserving
a nested `.git` directory. The inventory records the exact upstream blob IDs.
The public upstream history can be fetched independently. The owner-authorized
GitHub fork is https://github.com/tarahamante-cloud/Amante-viewer-native, with
the native development source on `amante-branding`. GitHub identifies its
parent as `Aurora-Viewer/Aurora-Viewer`; the development branch preserves the
recorded baseline as its parent commit. The existing `Amante-viewer` repository
is retained separately for its existing development downloads.

## Safety and attribution

- Never use real grid credentials in automated tests or connect automated
  tests to a live grid.
- Keep upstream authorship, license notices, and accurate technical references.
- A downloadable CI package is not evidence of desktop runtime correctness.
- Release feeds and reporting services must not send Amante users or reports
  to Aurora's infrastructure.
