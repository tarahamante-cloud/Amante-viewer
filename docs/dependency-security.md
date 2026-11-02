# Native dependency remediation

The user authorized completing all six findings, including coordinated
font and Linux saved-login dependency replacements. These are deliberate
exceptions to the earlier branding-only reset, not renderer or texture fixes.
The website and published Alpha package are unchanged.

## Resolved dependencies

| Finding | Resolution |
| --- | --- |
| RUSTSEC-2026-0382, rustls-webpki 0.103.15 | Updated to 0.103.16; distrusted certificate intermediates rejected. |
| RUSTSEC-2026-0385, ctutils 0.4.3 | Initially patched to 0.4.7, then removed entirely when its saved-login dependency chain was replaced. |
| RUSTSEC-2026-0192, ttf-parser 0.25.1 | Removed, including transitive owned_ttf_parser/ab_glyph. Font and emoji metadata now use maintained skrifa/fontations, already used by egui. |
| RUSTSEC-2026-0370, zvariant 5.15.0 | Removed; Linux notifications/SLURL handoff use zbus 6's integrated wire types. |
| RUSTSEC-2026-0371, zbus_names 4.3.4 | Removed; name types now come from zbus 6. |
| RUSTSEC-2026-0372, zvariant_derive 5.15.0 | Removed; derive macros now come from zbus 6. |
| RUSTSEC-2026-0433, pulldown-cmark 0.13.5 | Additional high-severity finding discovered during verification; updated to 0.13.6. |

The four unmaintained libraries have **no patched standalone versions**.
Their replacements remove the old packages rather than suppressing advisories.

## Compatibility

- Embedded UI font files, family choices, weights, fallback fonts, collection
  face selection and bitmap color emoji are retained. A small `amante-fonts`
  crate adapts the maintained parser to the viewer's metadata needs. Actual UI
  text rendering remains egui's existing fontations renderer.
- Wayland client-side window decorations and title text remain enabled, using
  winit's supported FreeType/crossfont backend instead of ab_glyph. Windows
  and macOS window backends are unchanged.
- `amante-credentials` selects the same native Windows Credential Manager and
  macOS Keychain stores as before. Linux still uses the OS Secret Service with
  encrypted DH sessions, the same `service`/`username` lookup attributes and
  default collection. Only its transport changes to the maintained libdbus
  implementation. No stored account is deleted or migrated to plaintext.
- The viewer's existing password/MFA service names and account identifiers
  remain unchanged. Missing or unavailable credential stores fail explicitly;
  there is no disk-file credential fallback.
- Linux D-Bus calls use zbus 6's `block_on` at the same synchronous boundaries.
  Notification signal streams use async iteration on their existing worker
  thread; SLURL method timeouts and shutdown name release are retained.
- Rendering, world, texture, networking and wearable engine source stay
  byte-identical to the recorded clean Aurora import.

## Source inventory

`amante-upstream-manifest.json` retains the pre-migration source hashes.
`amante-security-manifest.json` separately records each changed file's baseline
and updated hash, plus the new dependency-adapter crates.
`scripts/verify-clean-upstream.py` accepts only the named dependency manifests,
font/emoji callers, Linux notification/SLURL callers and adapter additions.
It rejects renderer overrides and reintroduction of the retired dependencies.
Windows CRLF checkout normalization remains supported.

## Verification

- `python3 scripts/verify-clean-upstream.py amante-viewer`
- `python3 scripts/test-clean-upstream-security.py`
- `cargo +1.99.1 test --manifest-path amante-viewer/Cargo.toml --profile ci --locked -p amante-fonts -p amante-credentials`
- `bash scripts/test-native-credentials.sh` runs the real native Secret Service
  round trip and the viewer's actual Linux SLURL module on an isolated session
  bus and temporary keyring. It never opens a user's existing credential store.
- Full native compilation and desktop/live-grid behavior are separate evidence.
  Do not describe these changes as a new published Windows package.

Local verification passed the five font tests, the credential facade test,
the actual SLURL handoff/shutdown tests on a private D-Bus session, seven
source-inventory regression tests, and the complete native viewer binary
compile-check with Rust 1.99.1 and the locked dependency graph. The refreshed
dependency audit reports zero findings. Formatting checks on all changed Rust
files passed. This was a compile-check, not a new linked Windows package or a
live-grid runtime test.
The real Secret Service round trip could not run in this sandbox:
`gnome-keyring-daemon` refuses its restricted process capabilities. The script's
explicit `AMANTE_SKIP_NATIVE_STORE=1` mode runs only the bus tests and prints
that the credential-store test was skipped. CI installs gnome-keyring and runs
the full default mode; do not equate configuring that check with a passing
CI or native-store result.

## Advisory sources

- https://rustsec.org/advisories/RUSTSEC-2026-0192.html
- https://rustsec.org/advisories/RUSTSEC-2026-0370.html
- https://rustsec.org/advisories/RUSTSEC-2026-0371.html
- https://rustsec.org/advisories/RUSTSEC-2026-0372.html
- https://rustsec.org/advisories/RUSTSEC-2026-0382.html
- https://rustsec.org/advisories/RUSTSEC-2026-0385.html
- https://rustsec.org/advisories/RUSTSEC-2026-0433.html
