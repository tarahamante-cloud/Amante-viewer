# Security changes and remaining risks

This is an engineering record, **not a security certification**.
Baseline: `d555fb677004bee8fd9e7f12b0e33a8978e77d4c`.

## Verified source defects addressed

### Hostname verification bypasses

The baseline disabled hostname verification for simulator capabilities, asset
fetching, the private bridge client and simulator release-note retrieval.
The bypasses were present in `aurora-net/src/lib.rs`, `bridge.rs` and
`release_notes.rs`.

These clients now share a strict TLS builder. Public roots and Linden Lab's
bundled CA are retained; neither an invalid chain nor a certificate for another
hostname is deliberately accepted. The bridge and release-notes clients retain
their no-redirect policies.

Regression coverage uses an ephemeral local TLS certificate and loopback
listener. It distinguishes a matching trusted hostname from a mismatched
hostname and verifies that application data is not received in the latter case.

**Compatibility risk:** a simulator using a CA-signed but mismatched certificate
now fails securely. No live-grid test has established how often this occurs.
Do not restore a blanket bypass. A future compatibility solution needs trusted
endpoint identity/pins or a corrected server certificate, not a name taken from
an unauthenticated server response.

### Credential-bearing login redirects

The baseline assigned XML-RPC `next_url` without checking the destination.
The login HTTP client also used default HTTP redirects.

The selected login endpoint must now be HTTPS. Both HTTP and XML-RPC redirects
must stay on that endpoint's origin; userinfo, fragments, protocol downgrades,
host changes and port changes are rejected. Redirects remain bounded.
Reqwest error URLs are removed from login errors.

Local regression fixtures verify rejection before contacting an untrusted
destination, continued same-origin XML-RPC redirects, plaintext rejection
before connection, lookalike-origin rejection and secret-free request Debug
output. All credentials are synthetic; no live grid is involved.

**Compatibility risk:** plaintext or cross-origin custom-grid flows need
owner-approved secure configuration/design work. They are not silently allowed.

### Credential/reporting identity and accidental disclosure

Login request/response Debug output and crash OAuth authorization Debug output
are redacted. Passwords, MFA fields, raw login responses and OAuth tokens are
not printed by these formatters. Credential namespaces no longer reuse Aurora's.
Reporting endpoints and OAuth client identifiers have no upstream defaults;
unconfigured sends return the existing explicit configuration failure.
Browser draft creation returns no URL when the destination is unconfigured.

This is targeted hardening, not proof that every possible HTTP error or
third-party logging path is secret-free. The remaining all-client and
private-report/body review is still open.

### Updater trust boundary

Production update network clients are blocked independently of the feed URL.
The existing loopback fixture transport remains usable. Existing ZIP path,
size, checksum, staging and rollback logic was retained, and Amante's executable
and package names are recognized.

**Unresolved:** authenticated publisher metadata, pinned Amante public keys,
key rotation, anti-rollback guarantees, signing/release permissions and a safe
production activation procedure. There is no claim that checksums alone solve
update authenticity.

## Dependency review

Current OSV audit results were inspected rather than relying on old reports.
Compatible patches applied to the main lockfile:

| Dependency | Baseline | Patched | Finding |
|---|---|---|---|
| `ctutils` | 0.4.3 | 0.4.4 | RUSTSEC-2026-0385, constant-time mask/u128 behavior |
| `pulldown-cmark` | 0.13.4 | 0.13.5 | RUSTSEC-2026-0414, exponential wiki-link parsing |
| `rustls-webpki` | 0.103.15 | 0.103.16 | RUSTSEC-2026-0382, distrusted intermediates |

The separate fuzz lockfile also receives `rustls-webpki` 0.103.16. Cargo
reconciled its existing audio dependency graph during this targeted update.
No native dependency group was stripped to make the build pass.

The inventory Markdown renderer enabled strikethrough, task lists and tables,
not wiki links; its code-block renderer used default options. The vulnerable
wiki-link mode was therefore not found enabled in these source call sites.
The patch is still applied as defense in depth.

### Findings not resolved by a compatible patch

- OSV's `GHSA-qwgh-2vcv-g2f7` matches `block-buffer` 0.10.4. Its current advisory
  range spans versions before 0.12.1 and describes a caught panic corrupting
  an inline cursor. The locked 0.10.4 implementation instead has a separate
  `pos` field. Exposure is not established; this needs upstream/advisory
  clarification and a transitive crypto dependency upgrade assessment.
  It is not dismissed as fixed or proven exploitable.
- `ttf-parser` and the older `zbus_names` / `zvariant` / `zvariant_derive`
  families have maintenance advisories. Replacement/upstream migration needs
  compatibility work; “unmaintained” is not by itself proof of an exploit.
- Starter JavaScript packages have separate audit findings. They are not native
  viewer dependencies and were not falsely counted as viewer vulnerabilities.

The installed packaged `cargo-audit` could not parse a current RustSec CVSS4
entry. That run is **not** successful audit evidence. Replit's current dependency
scanner was used separately.

## Static/privacy scans

The initial SAST scan returned one medium secret-URI finding in the updater
tests. Inspection shows an intentional `name:password@github.com` rejection
fixture, not a real credential. Privacy scanning returned no findings.
Neither result proves the absence of vulnerabilities.

The post-change dependency scan no longer reports the three patched advisories
in the viewer or fuzz lockfiles. Five moderate native dependency advisory matches
remain as described above; no critical/high native dependency matches were
reported. Separate starter JavaScript findings remain outside the viewer.

The post-change SAST scan returned the same rejection fixture and an additional
medium SHA-1 warning in the source-preservation script. That hash reproduces
Git's upstream blob identifiers for comparison; it does not authorize downloads,
sign releases or protect credentials. Treat it as provenance bookkeeping, not a
cryptographic publisher-authentication mechanism. Privacy scanning again
returned no findings.

## Release gates

## Additional native boundary review

The retained media plugin process binds its control listener to loopback on an
ephemeral port and launches the selected executable with structured arguments,
not a composed shell command (`aurora-media/src/process.rs`). The Blender live
transport also binds `Ipv4Addr::LOCALHOST`, uses deadline/stop-aware reads and
retains its original transport implementation. These protections were not
removed to make Replit compilation work.

This inspection is not an authentication audit of every local peer, native
decoder, asset parser, external URL path or subprocess. Those remaining surfaces,
including plugin provenance and local-peer authorization, still need review
before security sign-off.

## Release gates

Before a production release: finish all-client/log/privacy review, resolve or
document remaining advisory exposure, complete multilingual branding/service
ownership review, validate Windows packaging, implement update authentication,
and perform the desktop checklist. Never use real credentials in automated
tests, upload private logs by default, or enable updates merely to clear a UI
error.
