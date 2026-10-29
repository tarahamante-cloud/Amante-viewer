# Windows validation record

## Evidence under collection

- Owner-controlled source repository: <https://github.com/tarahamante-cloud/Amante-viewer>.
- Native source and build support only were uploaded with owner approval;
  unrelated Replit starter apps and local/build data were excluded.
- Current verification run: <https://github.com/tarahamante-cloud/Amante-viewer/actions/runs/40803551100>.
- Source revision submitted: `703544fe274016bffdb032b839afc7521e05f797`.
- Immutable upstream baseline: `d555fb677004bee8fd9e7f12b0e33a8978e77d4c`.
- Baseline comparison: all 8,643 original files present and tracked; 8,606
  unchanged, 37 modified, no original assets changed.
- The first source-import run was superseded and cancelled when the complete
  harness and distribution-verification steps were added. A cancelled run is
  not a passing build or test result.
- The next run found a source-export omission before any test execution:
  a broad `target` directory ignore excluded an upstream test module. Restored
  that exact baseline file without edits, narrowed the ignore exception, and
  added a tracked-inventory gate before compilation in every job. The initial
  filesystem-only comparison did not prove the exported Git tree was complete.

| Evidence level | Current outcome |
|---|---|
| Source preservation | Passed Git blob comparison against immutable inventory |
| Windows native target | Build running; not yet a pass |
| Actual Windows ZIP and SHA-256 | Pending; no checksum is claimed |
| Extracted CLI, PE/resources, assets/licenses | Pending Windows package verification |
| Complete Windows unit/integration harness | Compiled on 17,174,360,064-byte Windows runner; 6,085 passed, 2 failed, 30 ignored |
| Interactive Windows desktop | Not exercised |
| Graphical updater/rollback fixture | Not exercised; passing updater unit fixtures would not replace this |
| Live-grid behavior | Not exercised; voluntary human-only |

The Linux job on the current run completed the all-target compile check, then
failed strict Clippy on a collapsible nested condition in the crash-report
window. A behavior-preserving let-chain fix is prepared locally. Formatting
and binary-only Clippy passed on that fix. The local full-workspace/all-target
Clippy attempt was killed under the same 8 GiB limit when checking the viewer
test target; that attempt is not a pass. The Windows jobs were left running to
collect their independent results before a combined CI correction.

The complete viewer test executable **ran successfully**: 5,083 passed, zero
failed, nine ignored. The loopback updater integration suite passed all ten
tests, including cancellation, corrupt archives, extraction rejection, failed
launch rollback and profile preservation. The entire workspace job was still
red: two inherited assertions expected `Aurora Viewer` and `AuroraViewer/`
instead of the runtime's intended Amante identity. Both expectations were
corrected to literal Amante strings, not weakened or replaced with the
implementation's own constants. The six version integration tests and targeted
currency request test passed locally after correction. These local results
are not represented as a passing rerun of the Windows workspace suite.

The original Windows evidence artifact is
<https://github.com/tarahamante-cloud/Amante-viewer/actions/runs/40803551100/artifacts/12868511080>.
It contains actual compiler output, memory capacity, per-binary exit codes and
logs. The nine ignored viewer tests and other ignored workspace tests are not
counted as passes. The release/package job from that run remains separate.

`feature-parity.csv` enumerates all 1,996 inventoried UI modules, menu call
sites, settings and shortcut/toolbar declarations. Each row has separate
source, Windows-build, Windows-package and desktop-status columns. Without a
successful package-verification report, the generator leaves Windows outcomes
unverified. With that report, it labels whole-target/distribution evidence
explicitly as **not a per-item test**. Desktop outcomes remain untested until
actually exercised; dynamic or platform-dependent UI still needs inspection.

## Remaining desktop evidence

Use [DESKTOP_TESTING.md](DESKTOP_TESTING.md), first in offline demo mode with
an isolated Amante profile. Record OS, GPU/driver, display scaling, locale,
source revision and actual package checksum. Do not use an Aurora profile.
Preserve per-item failures, not-applicable reasons and untested states.

The automated harness uses synthetic fixtures and outbound non-loopback
firewall restrictions on the test executables. It does not run ignored tests
or sign into a grid. A passing harness cannot establish real Windows keyring
isolation, notification/URL registration, graphics/audio/device behavior,
keyboard focus, every menu action, or the real update/relaunch handshake.
These require appropriate Windows desktop evidence. No live-grid test is
required to download a development package, and none is automated.
