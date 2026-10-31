# Amante native security review

Reviewed 2026-10-29. This is a source-level review with offline regressions,
**not security certification or desktop/runtime sign-off**. Production
executable updates remain disabled. The old report was truncated; this replaces
it with explicit evidence and remaining risks.

## Executable update authentication

See [AUTHENTICATED_UPDATES.md](AUTHENTICATED_UPDATES.md) for the signed format,
compiled public trust anchor, dual-quorum root rotation, expiry, persistent
anti-rollback state, bounded downloads and activation procedure.

Publisher authenticity is distinct from archive integrity. Production no longer
accepts unsigned GitHub release JSON/policies or a replaceable checksum sidecar
as authority. Package hashes and maintenance policy come from signed snapshots.
The owner's public keys have **not** been supplied: the compiled empty root and
false production flag intentionally prevent activation. No private key was
committed or generated for production. Merely setting URLs cannot enable code
replacement. Existing loopback fixtures, ZIP validation and transactional
filesystem/profile rollback remain available.

## All-client credential and TLS review

Reviewed the production HTTP constructors and their consumers across
`aurora-net`, `aurora-viewer`, `aurora-voice` and `aurora-updater`. Bare
`Client::new()` instances in offline test modules are not production clients.

| Client / consumers | Credentials, boundary and disposition |
| --- | --- |
| `aurora-net/http_security.rs`, `login.rs` | Password/MFA login requires HTTPS. HTTP redirects and XML-RPC `next_url` preserve exact HTTPS origin (host, effective port and protocol); no userinfo/fragments. Certificate chain **and hostname** remain verified, including Linden CA use. Debug output hides login secrets. |
| `aurora-net/lib.rs` capability client and `bridge.rs` | Simulator roots retained, HTTPS required, credential POST redirects restricted to the initial origin. Capability URLs/agent sessions are bearer-like secrets. No blanket hostname bypass. |
| `aurora-net/lib.rs` asset client | Same strict roots/hostname and HTTPS policy; retains GET decoding/pool settings. Cross-origin redirects are now rejected rather than forwarding capability URL queries or accepting an unsigned destination. |
| Inventory, uploads, event queues, stats, abuse reports, currency and media capability calls | Use shared network clients; redirect policy therefore covers session-bearing request bodies, not just Authorization headers. `currency.rs` uses only the known Linden helper for supported grids; no raw parameters/answers are logged. Statistics explicitly omit reusable session IDs. Grid-supplied upload capabilities remain delegated endpoints of the selected grid, not trusted Amante service endpoints. |
| `aurora-voice/session.rs`, viewer `voice.rs` / `app.rs` | Voice signaling consumes the strict shared capability client, not a separate permissive client. SDP/voice session bodies cannot follow cross-origin POST redirects. Media transport/codec parsing is a separate native attack surface below. |
| Viewer `media/openid.rs` | Token exchange uses the shared capability client. Cookies/tokens are not deliberately logged; URI-bearing transport errors are removed at the central log sink. The selected grid still supplies its OpenID endpoint; this is not permission to reuse another grid's session. |
| Viewer `snapshot/profile.rs` | Fixed a cookie-forwarding flaw: the server-supplied storage upload receives the signed form and PNG **without** profile/OpenID cookies unless it is the exact profile HTTPS origin. The finishing redirect must return to that origin. Cookie acquisition uses same-origin HTTPS redirects; plaintext profile posts are refused. |
| Viewer `bug_report/github.rs`, `crash/github.rs` | Explicit reporting configuration required; fixed GitHub device/refresh endpoints and coherent owner issue destination; HTTPS, no redirects, bounded responses, no automatic create retries. Tokens use the Amante credential namespace and redacted Debug. No arbitrary redirect can receive a GitHub bearer token. OAuth activation is a separate task, not enabled by this review. |
| Updater `feed.rs`, `package.rs`, `media_pack.rs` | No account credentials/cookies attached. Fixed host HTTPS allowlist, bounded redirect count/metadata, signed executable authority and pinned media integrity. Updater failures log classification, not raw signed CDN URLs. Local fixtures use explicit loopback HTTP with no system proxy. Media distribution remains independently gated. |
| Viewer `grids.rs`, `grid_status.rs`, `ui/news.rs` | Public discovery/status/news GETs, no grid password, cookie or OAuth headers attached. Discovery is not a credential transport; HTTP discovery does not authorize HTTP login. |
| Viewer `link_preview.rs`, `gif_search.rs` | Separate unauthenticated clients; previews have URL/redirect checks, timeouts and body limits. They never inherit grid/OAuth headers. Public requests can disclose the user's IP to the requested service; this is a privacy/consent consideration, not credential forwarding. |
| Viewer `snapshot/profile.rs` storage form and embedded CEF browser | Storage form fields are purpose-scoped upload authorization, intentionally sent to the profile service's chosen HTTPS storage endpoint. Browser cookies have their own origin/domain handling; browser plugins are executable trusted components, not a sandbox proven by this review. |

### Explicit strict-TLS/custom-grid compatibility conflict

Some Second Life simulator configurations historically depended on accepting
certificates for a different hostname; some custom grids use HTTP capabilities,
private/self-signed CAs or cross-origin redirecting capability/CDN endpoints.
Those configurations may fail now. The simulator trust store is still the
retained Linden simulator roots, **not** a universal custom-grid CA store.
Profile upload completion redirects to other origins are also refused.

This is an unresolved compatibility limitation, deliberately reported instead
of silently disabling TLS verification. Required grid-owner remedies are
matching hostname/SAN certificates, HTTPS endpoints, direct capability URLs
and same-origin credential redirects. A future custom-grid trust feature must
bind owner-approved roots/endpoint delegation to one selected grid, show clear
consent, preserve hostname checks, and never forward existing-grid secrets to
new origins. No global `danger_accept_invalid_certs`, hostname bypass or
automatic trust-on-redirect was added. Live-grid compatibility is unverified.

## Logs, credentials and private reports

- Central viewer logging now removes full HTTP/WebSocket URLs (including
  arbitrary capability paths, userinfo and query secrets), not just `/cap/UUID`.
  Records naming password/token/cookie/authorization/session credential fields
  are suppressed before console, current/session files, capture and recent-ring
  sinks. The existing bare-capability redaction remains as defense in depth.
- This is **not a claim that private logs contain no personal information**:
  panic text, object names, user-entered descriptions, filesystem paths and
  conversation context can be free-form. Transcripts intentionally remain local
  user data; they are not suitable for automatic publication.
- Public diagnostics retain the closed-vocabulary sanitizer in
  `crash/privacy.rs`; arbitrary error fragments are never made public merely by
  replacing UUIDs. Crash reports default to no excerpt; field inclusion and
  screenshots require review/consent.
- `bug_report/files.rs` separates `public/` from `private-do-not-upload/`.
  Raw logs/traces and unchecked screenshots stay in the private directory.
  Public issue creation uses reviewed markdown, not recursive upload of a log
  directory. Reports/transcripts are not automatically uploaded by this work.
- Credential storage uses Amante's namespace; tests use synthetic credentials.
  No real grid login or live-grid traffic was used in regressions.
- Local data privacy still depends on OS account/file permissions and what the
  user chooses to share. The log redactor is defense in depth, not a public
  disclosure filter or a guarantee against resident-controlled PII.

## Native media, local peers and untrusted input

| Surface | Existing protection / residual finding |
| --- | --- |
| Media process (`aurora-media/src/process.rs`) | Ephemeral IPv4 loopback listener; structured executable/port arguments, launch/heartbeat/exit deadlines, no composed shell command. **No peer authentication:** the first local TCP connection is accepted. A local process can race the plugin, inject protocol messages or exhaust the unbounded XML input/received queue. Protocol-compatible peer authentication and message/resource bounds are release blockers for a hostile-local-peer threat model. |
| Media shared memory (`aurora-media/src/shm.rs`) | Random segment identity, Unix 0600 creation, owned mappings and checked size conversion. It is not isolation from a malicious trusted plugin/same-user process. Windows mapping ACL behavior needs Windows runtime validation. |
| Blender live (`aurora-viewer/src/blender_live/transport.rs`, `protocol.rs`) | Loopback binding, protocol magic, bounded frames, supported-joint checks, timeout/stop-aware reads. **Magic is not authorization:** any local client can speak the protocol, read rig information or submit scene/shape data while live mode is enabled. No secret pairing or OS peer identity check exists. Retained rather than silently breaking the addon protocol; require explicit pairing/authentication before claiming multi-user/local-peer safety. |
| External URLs (`external_url.rs`, `slurl_handoff.rs`, app/crash platform output) | Incoming SLURLs use command classification/throttling and do not directly teleport through click-only commands. Both OS browser-output sinks now allow only HTTP/HTTPS/mail links; file/data/javascript/shell/custom OS handlers are blocked there. Explicit local-file manager actions remain separate. SLURL handoff itself is loopback IPC, not cryptographic sender identity. |
| Subprocesses (updater, media, Blender installer, external script editor) | Executable plus argument vectors, no remote text spliced into a shell. Blender installation uses background mode and `--disable-autoexec`. User-configured editors/plugins are intentional local executable trust decisions; verifying a plugin's bytes does not prove its CEF/GStreamer/native code safe. Updater helper/launch paths are prepared locally and retain rollback on failed startup. |
| Assets (`aurora-assets`, `aurora-llsd`, Blender/image import) | Mesh decompression has output bounds; J2K checks dimensions/pixels/components before OpenJPEG; animation bounds joints/keys/duration. Existing offline malformed-input cases and fuzz targets cover LLSD, J2K, meshes, sounds, animations, messages and other formats. These are not proof against every decoder/FFI bug. Browser codecs, Opus/OpenJPEG and font/image parsers remain reachable native attack surfaces. Fuzzing/desktop sanitizer campaigns were not run in this environment. |

Do not claim that loopback binding, process separation or protocol magic
constitute local-peer authentication. No live plugin, Blender or GPU runtime
was exercised here, and plugin automatic downloads were not activated.

## Dependency advisory investigation

The dependency scan on 2026-10-29 reported **five moderate native matches**
and no critical/high native matches. Its seven high findings belong to the
separate root JavaScript lockfile, not the native viewer; they are not dismissed
as safe, but are outside this native task. Static scan found two medium findings:
the intentional credential-URL rejection fixture and SHA-1 used only to reproduce
Git blob provenance IDs in `scripts/verify-viewer-preservation.mjs`. Neither is a
real embedded credential or publisher-authentication mechanism. Privacy/dataflow
scan returned no findings; that is not proof of absence.

### `block-buffer`: scanner match does not match the affected range

Scanner finding: `GHSA-qwgh-2vcv-g2f7@block-buffer-0.10.4`.
[Publisher advisory](https://github.com/RustCrypto/utils/security/advisories/GHSA-qwgh-2vcv-g2f7)
explicitly lists **>=0.11.0,<0.12.1**, fixed in **0.12.1**. It concerns a
compression/generator panic corrupting an **inline cursor**, followed by reuse
after catching the panic.

Both native lockfiles retain `block-buffer 0.10.4` and patched `0.12.1`.
`cargo tree -i block-buffer@0.10.4` traces the older line through digest 0.10,
HMAC 0.12, MD5/SHA1 and `rtc-crypto` / WebRTC voice. Inspection of downloaded
0.10.4 source confirms a **separate `pos: u8` field** rather than the affected
inline cursor; `digest_blocks` passes only the block buffer to compression.
Thus the specific reported inline-cursor corruption does not apply to 0.10.4;
0.12.1 is the fixed line. Retain this as a documented scanner range mismatch,
not an assertion that all old digest code is vulnerability-free. No unsafe
cross-major override was introduced and no scanner suppression hides the match.

### Maintenance advisories: unresolved dependency replacement work

| Advisory | Locked dependency / route | Disposition |
| --- | --- | --- |
| RUSTSEC-2026-0192 | `ttf-parser 0.25.1`, direct font use and `owned_ttf_parser -> ab_glyph -> sctk-adwaita -> winit` | Unmaintained. `skrifa`/fontations migration requires font metrics/shaping and GUI-window compatibility testing; changing only the direct dependency leaves the transitive copy. No known exploit is established by this maintenance advisory, but untrusted font handling remains an exposure. |
| RUSTSEC-2026-0371 | `zbus_names 4.3.4` through zbus 5.19 / keyring 4.2 / secret-service | Types move into zbus 6. Requires coordinated Linux credential-backend migration, not a blind lockfile bump. |
| RUSTSEC-2026-0370 | `zvariant 5.15.0`, same Linux credential stack | Wire types move into zbus 6; preserve existing secret-store access and test missing/locked keyrings. |
| RUSTSEC-2026-0372 | `zvariant_derive 5.15.0`, same dependency family | Derive macros move into zbus 6; migrate with parent stack. |

These four maintenance advisories remain explicitly unresolved. They are not
silently waived and were not counted as exploitable critical/high findings.
The previously installed `cargo-audit` could not parse a CVSS4 advisory; that
failed tool invocation is not successful audit evidence. This review used the
current dependency scanner, publisher range and local dependency/source
inspection instead.

## Verification and release gates

Validation on 2026-10-29 (Rust 1.99.1, CI profile, offline locked dependencies):

```
cargo test --profile ci --locked --offline \
  -p aurora-updater -p aurora-version -p aurora-net -p aurora-assets -p aurora-media
cargo check --profile ci --locked --offline -p aurora-viewer --bin aurora-viewer
```

The complete smaller-crate pass executed 563 tests successfully; the final
additional corrupt-state and signed-channel regressions brought covered
smaller-crate tests to 565 (the affected updater/version suites were rerun).
The normal viewer binary compile-check passed with existing platform-specific
dead-code warnings. No Windows executable build or desktop runtime is claimed.
Strict all-target Clippy passed for `aurora-updater`, `aurora-version` and
`aurora-net` with `-D warnings`.

The executable-update suite exercises signatures, root/release-role separation,
dual-key rotation, duplicate-signature thresholds, expiry/future issuance,
sequence equivocation/rollback, persistent version/time floors, altered offers,
unsafe URLs, invalid maintenance policy and corrupt/unwritable state. Existing
local download/extraction/cancellation/installation/profile rollback fixtures
are retained. HTTPS loopback tests cover hostname mismatch, HTTP/XML-RPC
redirect rejection, same-origin login and capability POST non-forwarding.

Viewer-only profile-cookie regressions are part of the native viewer harness;
do not claim execution solely because the normal application compile-check
passes. Native Windows/runtime/GPU tests still require the desktop checklist
and sufficient build memory.

Before a production security sign-off:

1. Owner public trust keys, independently reviewed signing custody, valid
   signed publication and explicit release activation; never just a feed URL.
2. Authenticate local media/Blender peers and bound media protocol resources.
3. Migrate/explicitly accept the font and Linux keyring maintenance risks.
4. Resolve the specific grid's strict-TLS compatibility; never reintroduce a
   blanket hostname bypass or forward secrets to untrusted redirect origins.
5. Complete Windows desktop/plugin/decoder validation and untrusted-parser
   fuzz/sanitizer testing. Preserve known runtime-evidence limitations.
