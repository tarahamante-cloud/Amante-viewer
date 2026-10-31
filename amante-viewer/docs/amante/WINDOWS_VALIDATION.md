# Windows validation record

## Latest checked package: update-notification regression corrected

The failed run shown in the user's screenshot is **#12**, run
[40870771402](https://github.com/tarahamante-cloud/Amante-viewer/actions/runs/40870771402),
source `61144881cee64ef66c149a2dbb8ac5ae959603fd`. Linux and the Windows
application/package jobs passed. The Windows viewer test
`ui::update::tests::required_offer_has_no_skip_or_later_and_escape_cannot_close_it`
failed with `forbidden choice: Later`; the viewer result was 5,086 passed,
one failed and nine ignored. This was a real required-update UI regression,
not a Windows compiler or packaging failure.

The owner's newer source already contains the correction:
the notification-only `State::Offer` branch is selected only when `!required`.
Required offers therefore retain their existing non-dismissible behavior.
The original test was retained, not skipped or weakened.

All three jobs passed in the newer **#13**, run
[40882122452](https://github.com/tarahamante-cloud/Amante-viewer/actions/runs/40882122452),
source `824d7dd8fb179673c49efd50a984de6d1e3f512e`. The historical failed
run remains red; rerunning its old source would not apply the fix.

| Evidence level | Verified result / limitation |
|---|---|
| Source | Exact packaged revision checked out separately; immutable baseline has 8,643 files, 8,521 unchanged, 122 modified, none missing/untracked and no original assets changed |
| Linux build | Complete root workflow job passed |
| Windows build / package | Complete native x64 release and extracted CLI/resource/license verification passed on Windows NT 10.0.26100.0 |
| Windows offline harness | 6,099 passed, zero failed, 30 ignored across 16 executables; compilation passed on a 17,174,360,064-byte runner |
| Viewer harness | 5,088 passed, zero failed, nine ignored; the exact previously failing required-offer test reports `ok` |
| Profile / updater fixtures | Credential-namespace isolation and local rollback/profile-preservation fixtures passed; actual Windows keyring/profile and graphical update/relaunch remain unverified |
| Distribution | Independently downloaded original ZIP, matching checksum sidecar; 32 archive entries; x64 `amante-viewer.exe` is 164,994,560 bytes |
| Per-area matrix | All 1,996 source/build/package rows bind to this exact revision and ZIP; every interactive desktop-runtime field remains `not runtime-tested` |
| Desktop / live grids | Not exercised or claimed; use the human desktop checklist, offline first, with live-grid testing optional and human-only |

- Package: `Amante-Viewer-v0.0.0+17.g824d7dd8f-windows-x64.zip`.
- Inner ZIP SHA-256:
  `a3bba77864a54b31ad09715e792d6d7bdcbba4028ba4c1c3da027d37a75dc4fb`.
- Package artifact: <https://github.com/tarahamante-cloud/Amante-viewer/actions/runs/40882122452/artifacts/12903241830>.
- Test artifact: <https://github.com/tarahamante-cloud/Amante-viewer/actions/runs/40882122452/artifacts/12901539675>.

These receipts were checked against an exact checkout of the owner's packaged
source, not assumed from this workspace's independently evolving source tree.
The older packages and scan reports below concern their own exact checksums;
they are not malware-scan evidence for this new package. This remains an
unsigned development ZIP, not an authenticated release or safety certification.
Do not bypass a browser or antivirus block.

## Earlier verified branding build — desktop validation pending

The user requested Amante across every product-owned landing screen, menu and
other visible surface after confirming the direct download worked. Source now
includes that expanded branding, branded bundled news, project links and
distribution README. Existing content, compatibility identifiers and required
upstream legal notices are not rewritten.

The updated package is
`Amante-Viewer-v0.0.0+14.gca32d4292-windows-x64.zip`, from source
`ca32d42921c3c0c3a2d6b63791c37d753f2eb5d9`.

- Build: <https://github.com/tarahamante-cloud/Amante-viewer/actions/runs/40825559436>.
- Windows artifact: <https://github.com/tarahamante-cloud/Amante-viewer/actions/runs/40825559436/artifacts/12877593258>.
- Inner ZIP SHA-256:
  `cba3e43ae3df90b63ecca5a572f3bfa873285f2d0eaed4d04e89916ca2fd741b`.
- Offline evidence: <https://github.com/tarahamante-cloud/Amante-viewer/actions/runs/40825559436/artifacts/12877497096>.

| Evidence level | Verified result / remaining limitation |
|---|---|
| Source | All 8,643 original files retained and tracked; 8,525 canonical blobs unchanged, 118 modified, no original assets changed |
| Linux build | Formatting, all-target checks/Clippy, targeted offline tests, normal native binary build and CLI passed |
| Windows build | Complete optimized native x64 application built successfully |
| Full Windows offline harness | 6,094 passed, zero failed, 30 ignored across 16 executables; compilation passed on a 17,178,693,632-byte runner |
| Viewer offline harness | 5,086 passed, zero failed, nine ignored; the three earlier regressions below now pass |
| Distribution | Original inner ZIP independently downloaded and checksum matched; archive contains 31 entries, including the 165,193,728-byte x64 `amante-viewer.exe` and Amante README |
| Identity / licenses / assets | Extracted `--version`, x64 PE, Amante ProductName/FileDescription/OriginalFilename and packaged document/license comparisons passed; embedded original assets preserved |
| Profiles / updater | Offline namespace/path and loopback update/rollback fixtures passed; real keyring, interactive profile isolation and graphical updater/relaunch still unverified |
| Per-area evidence | All 1,996 feature CSV rows bind to this exact source and package; whole-target build/package checks are not per-feature runtime tests |
| Desktop runtime | Not verified: native GPU startup, window/taskbar/notification identity and interactive menu/tool/settings/shortcut checks still require the human desktop checklist |

The original ZIP was also scanned without executing or repacking it:
<https://github.com/tarahamante-cloud/Amante-viewer/actions/runs/40830009993>.
Defender updated signatures successfully, returned zero, reported no detection
and confirmed the same package checksum afterward. This is one scanner's
result, **not** proof of safety, code signing, update authenticity or Google
Safe Browsing clearance. Do not bypass a browser or antivirus block.

Earlier packages below do **not** contain this expanded branding. Their
checksums and results remain evidence only for their own revisions. The current
CSV explicitly leaves every desktop-runtime field as `not runtime-tested`.

### Branding candidate with failed regressions

Run [40822636228](https://github.com/tarahamante-cloud/Amante-viewer/actions/runs/40822636228)
tested source `681f42f1b06d260f751c1426f91e6e25c6dd5cd4`.

- **Source:** all 8,643 baseline files retained; no original assets changed.
- **Build:** Linux checks, all-target Clippy and normal binary build passed.
  The complete Windows test harness compiled on a runner reporting
  17,174,360,064 bytes of memory. The separate optimized Windows build was
  superseded before completion, so this is not a release-build pass.
- **Package:** no replacement ZIP is certified by this run.
- **Offline runtime:** all 16 test binaries ran: 6,091 passed, 3 failed and
  30 were ignored. The viewer alone had 5,083 passed, 3 failed and 9 ignored.
  Failures were the old demo-group name assertion and two inventory ordering
  checks: support-folder sorting had not recognized the new `#Amante` name.
- **Desktop runtime:** interactive Windows/GPU checks remain unverified.

Source `ca32d42921c3c0c3a2d6b63791c37d753f2eb5d9` corrects those causes,
retains legacy `#Aurora` support-folder priority without renaming user data,
and adds coverage for both folder names. The corrected full workflow is
[40825559436](https://github.com/tarahamante-cloud/Amante-viewer/actions/runs/40825559436);
all three corrected workflow jobs passed. This result is not inferred from
the earlier failed candidate.

## Successful automated run — 2026-10-29 UTC

Root workflow: <https://github.com/tarahamante-cloud/Amante-viewer/actions/runs/40806913429>.
All three jobs passed: Linux, Windows release packaging, and the complete
Windows offline harness. Tested source:
`755da7b22999cb255c22d41183882c7f08724cdd`.

**Historical package, superseded by the branding build above:**

- Download: <https://github.com/tarahamante-cloud/Amante-viewer/actions/runs/40806913429/artifacts/12869344616>.
- Inner ZIP: `Amante-Viewer-v0.0.0+6.g755da7b22-windows-x64.zip`.
- SHA-256 of the **inner versioned ZIP**, not GitHub's outer artifact ZIP:
  `4d2664eca934e31e35dd7d56c82f5b77cdc9829a697f593067a2defb19a8f55a`.
- Test evidence: <https://github.com/tarahamante-cloud/Amante-viewer/actions/runs/40806913429/artifacts/12868329829>.
- Artifacts have 14-day retention; they are not a published release.

| Evidence level | Verified outcome / remaining limitation |
|---|---|
| Source preservation | 8,643 baseline files present and tracked; canonical comparison: 8,605 unchanged, 38 modified, no original assets changed |
| Linux | Formatting, all-target check and Clippy, targeted offline security/updater tests, native build and CLI passed |
| Windows native application | Complete release build passed on Windows NT 10.0.26100.0 |
| Windows full harness | Compiled on 17,174,360,064-byte runner; **6,090 passed, zero failed, 30 ignored**, across 16 executables |
| Viewer harness | **5,086 passed, zero failed, nine ignored** |
| Profile isolation | Default config/cache/media/log path separation, isolated unit-test paths and separate literal credential service names passed; real keyring/profile behavior not exercised |
| Local updater | Ten unit tests and ten loopback integration tests passed, including rollback/profile-preservation cases; graphical update/relaunch still untested |
| Distribution | Actual download, independent inner ZIP checksum, archive integrity, 29 entries, `amante-viewer.exe` present (165,228,544 bytes) |
| Windows identity | Extracted `--version`, x64 PE and ProductName/FileDescription/OriginalFilename checks passed |
| Assets/licenses | Packaged license/doc/source files matched their build source; original embedded assets canonically preserved; no graphics rendering inferred |
| Interactive desktop | Not exercised: GPU, menus/settings/shortcuts, dialogs, real keyring, devices and graphical updater fixture remain pending |
| Live grids | Not exercised, never automated; voluntary human-only |

The executable imports `MSVCP140.dll`, `VCRUNTIME140.dll` and
`VCRUNTIME140_1.dll`. This unsigned development ZIP does not include Microsoft's
runtime. See [runtime prerequisites](WINDOWS_BUILD.md#windows-runtime-prerequisites).
The hosted developer image already has it; neither clean-machine launch nor
Vulkan graphics is certified by successful `--version`.

The original Windows source report used raw checkout bytes and therefore
counted CRLF conversions as edits. Retained it unchanged, then replayed the
exact build commit with Git-normalized comparison locally. All 8,643
Windows-recorded blob identities matched that checkout or its CRLF form, with
no unexplained source differences; canonical counts are those above. The
replayed `feature-parity.csv` attributes all 1,996 rows to this exact build
revision and package SHA, explicitly **not** to per-feature runtime tests.

After this run, only documentation/evidence tooling and CI wiring changed;
native product source was not changed. Seven local synthetic regression tests
cover checkout line endings, real binary edits, missing/untracked source,
distinct evidence levels, mismatched package revisions and post-build native
edits. Source-only invocations intentionally reset CSV Windows evidence;
package attribution requires a clean native checkout of the exact built
commit. The post-run evidence-tool changes are not claimed to have executed
in the successful Windows run.

## First completed Windows build

- Owner-controlled source repository: <https://github.com/tarahamante-cloud/Amante-viewer>.
- Native source and build support only were uploaded with owner approval;
  unrelated Replit starter apps and local/build data were excluded.
- First package/test run: <https://github.com/tarahamante-cloud/Amante-viewer/actions/runs/40803551100>.
- Source revision submitted: `703544fe274016bffdb032b839afc7521e05f797`.
- Immutable upstream baseline: `d555fb677004bee8fd9e7f12b0e33a8978e77d4c`.
- Canonical baseline comparison: all 8,643 original files present and tracked;
  8,607 unchanged, 36 modified, no original assets changed.
- The first source-import run was superseded and cancelled when the complete
  harness and distribution-verification steps were added. A cancelled run is
  not a passing build or test result.
- The next run found a source-export omission before any test execution:
  a broad `target` directory ignore excluded an upstream test module. Restored
  that exact baseline file without edits, narrowed the ignore exception, and
  added a tracked-inventory gate before compilation in every job. The initial
  filesystem-only comparison did not prove the exported Git tree was complete.

| Evidence level | Earlier-run outcome |
|---|---|
| Source preservation | Passed Git blob comparison against immutable inventory |
| Windows native target | Passed release build on Windows runner |
| Actual Windows ZIP and SHA-256 | Downloaded and independently hash-verified; see package below |
| Extracted CLI, PE/resources, assets/licenses | Passed package verification on Windows NT 10.0.26100.0 |
| Complete Windows unit/integration harness | Compiled on 17,174,360,064-byte Windows runner; 6,085 passed, 2 failed, 30 ignored |
| Interactive Windows desktop | Not exercised |
| Graphical updater/rollback fixture | Not exercised; passing updater unit fixtures would not replace this |
| Live-grid behavior | Not exercised; voluntary human-only |

The Linux job on this earlier run completed the all-target compile check, then
failed strict Clippy on a collapsible nested condition in the crash-report
window. A behavior-preserving let-chain fix was subsequently applied. Formatting
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

## First actual development package

- Download: <https://github.com/tarahamante-cloud/Amante-viewer/actions/runs/40803551100/artifacts/12868593402>.
- Inner ZIP: `Amante-Viewer-v0.0.0+4.g703544fe2-windows-x64.zip`.
- SHA-256 of that **inner ZIP**, not GitHub's surrounding artifact ZIP:
  `60a24588b151b462c2835cc1f3ad6a8c3fe04de2b753d8c97c4031c0bcdbcb86`.
- Source revision: `703544fe274016bffdb032b839afc7521e05f797`.
- The release build took approximately 39 minutes; its Windows job was green,
  although the overall run was red from the separately documented Linux lint
  and two stale branding assertions.
- Downloaded the actual artifact independently, verified its inner checksum
  against both the SHA sidecar and Windows package report, checked ZIP
  integrity, and confirmed all 1,996 feature rows retain distinct
  source/build/package/desktop evidence. The archive contains 28 entries;
  `amante-viewer.exe` is 165,217,792 bytes.
- Windows verification executed the extracted CLI successfully and checked
  x64 PE headers, product/description/original-filename resources, install
  metadata, dictionary/emoji licenses and all packaged Amante documentation.
- Import-table inspection also found `MSVCP140.dll`, `VCRUNTIME140.dll` and
  `VCRUNTIME140_1.dll`. The package requires the supported x64 Microsoft Visual
  C++ Redistributable; the hosted developer image already has it. A clean
  Windows machine and a working Vulkan graphics driver remain unverified.

The initial Windows raw-byte source report classified checkout CRLF conversion
as modification. This was an evidence-tool limitation, not thousands of native
code/asset edits. Replayed the exact build revision locally with Git-normalized
comparison and checked **every one of the 8,643 Windows-recorded source blobs**
against that checkout or its CRLF form: no unexplained differences. Canonical
result: 8,607 unchanged, 36 modified, zero missing/untracked baseline files,
zero changed assets. The original raw report is retained, not overwritten;
the canonical report and replay proof are separate generated evidence. Added
regression fixtures for CRLF, real binary changes, missing/untracked source and
separate build/package/runtime attribution.

## Correction run

The branch `windows-validation` contains the identity assertion and lint fixes,
plus explicit no-I/O profile-path and credential-namespace regression tests.
The current full run is
<https://github.com/tarahamante-cloud/Amante-viewer/actions/runs/40806913429>
at source `755da7b22999cb255c22d41183882c7f08724cdd`. All three jobs passed.
Do not apply its results to
the earlier ZIP or substitute an unbuilt source revision for the packaged one.

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
