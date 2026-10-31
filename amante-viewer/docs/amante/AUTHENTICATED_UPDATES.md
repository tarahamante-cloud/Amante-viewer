# Authenticated Amante executable updates

## Shipped state: disabled, not production-authorized

`crates/aurora-updater/src/trust-root.json` is the **compiled public trust
anchor**, not a downloaded file or an environment override. Its empty key sets
and expired timestamp deliberately fail validation. The owner has not supplied
public signing keys. No usable production key has been invented, copied from
Aurora, or generated with private material in this repository.

`PRODUCTION_UPDATES_READY` remains false. Changing a URL does not activate
updates; changing the flag without valid pinned keys also fails closed.
Notification-only discovery remains separate and cannot download/install code.
This implementation is not evidence that a release is authorized for production.

## Wire contract

Serve at the compiled `AMANTE_RELEASES_API` endpoint a UTF-8 JSON object:

```
{
  "roots": [ /* complete signed root rotation chain after the compiled root */ ],
  "releases": {
    "signed": "<exact UTF-8 JSON string containing the release snapshot>",
    "signatures": [{"key": "<key ID>", "signature": "<128 hexadecimal digits>"}]
  }
}
```

All signatures use Ed25519, via `ring`. The signed message is the concatenation
of the domain below and the **exact decoded UTF-8 bytes** of `signed`. Do not
canonicalize or reserialize that string between signing and verification.
Outer whitespace/signature order does not affect the signed message.

| Role | Domain prefix, including final newline |
| --- | --- |
| Root | `Amante update root v1\n` |
| Release snapshot | `Amante update releases v1\n` |

The root shape is the compiled file's shape: schema 1, product
`amante-viewer`, positive integer `version`, Unix-seconds `expires`,
`root_threshold`, `release_threshold`, and separate `root_keys` /
`release_keys` objects mapping IDs to 32-byte public keys as 64 hex digits.
Thresholds must be positive and no greater than the number of distinct keys.
Aliases for the same key cannot inflate a threshold. At most 16 keys per role.

Root rotation:

1. Increment the root version by exactly one.
2. Sign the new root's exact payload with the **old root quorum and new root
   quorum**. Both thresholds are checked independently.
3. Publish the full chain from the compiled anchor in every bundle. Returning
   clients require an unchanged payload prefix matching their persisted chain.
4. Sign the snapshot with the final root's release keys, and set its
   `root_version` accordingly. Old release keys no longer authorize snapshots
   unless explicitly retained in the new root.

Keep offline root keys separate from online release keys. A 2-of-3 offline
root quorum is recommended; do not keep all root shares on the release runner.
The code does not contact a key server, load a private key or implement signing.
Use the owner's offline/HSM signing process, exporting signatures only.
Keep the compiled anchor stable while rotating its delegations; replacing the
anchor itself requires a separately reviewed trust-state migration.

Snapshot payload:

```
{
  "schema": 1,
  "product": "amante-viewer",
  "root_version": 1,
  "sequence": 1,
  "issued": 0,
  "expires": 0,
  "releases": [{
    "version": "1.0.0",
    "platform": "windows-x64",
    "notes": "Reviewed release notes",
    "url": "https://github.com/OWNER/REPO/releases/tag/v1.0.0",
    "archive": {
      "name": "Amante-Viewer-v1.0.0-windows-x64.zip",
      "size": 123,
      "browser_download_url": "https://github.com/OWNER/REPO/releases/download/v1.0.0/Amante-Viewer-v1.0.0-windows-x64.zip"
    },
    "sha256": "<64 hexadecimal digits>",
    "minimum_supported_version": null,
    "cache_invalidation_before": null,
    "settings_reset_before": null
  }]
}
```

The zero validity timestamps above are **not a valid publication**. Use current
Unix seconds and an expiry no more than seven days after `issued`. Refresh
metadata with a new global sequence even if package bytes are unchanged.
Do not reuse a sequence for different snapshot bytes. Stable and beta targets
belong in the same global snapshot; build-metadata versions are not releases.
Size, SHA-256, version, platform, notes, link and destructive maintenance policy
are authenticated together. GitHub's unsigned release JSON/policy/checksum
sidecars are used only by the retained explicit loopback fixtures.

Bounds: 1 MiB bundle, 100 targets, 64 sequential rotations, 32 signatures per
envelope, 2 GiB compressed archive; existing extraction bounds remain intact.
The wire structs reject unknown fields. Duplicate platform/version targets are
rejected. Only Windows x64 executable replacement is implemented.

## Anti-rollback and installation

Trust state lives in Amante's local application-data `update-trust/state.json`,
separate from cache/settings and updater staging. It stores signed root proofs,
the greatest observed snapshot sequence/hash, last successful wall time, and
the greatest offered version per channel. This advances on **authenticated
observation**, not successful installation. A failed installation or choosing
Later must not let a feed serve an older authenticated history.

- Reject older sequences, conflicting same-sequence payloads, root downgrades,
  per-channel target downgrades and wall-clock regression.
- Repeated identical snapshots are allowed only while valid. Reject expired or
  future-issued snapshots and an expired final root.
- Installed-version comparison uses SemVer precedence, ignoring build metadata;
  no older executable is offered even on a fresh state directory.
- A private authorization seal binds the exact offered release. The download
  checks it against durable state and time before fetching, immediately before
  extraction, and again when the user selects Restart. Metadata supersession,
  expiry or offer mutation aborts.
- Package SHA-256 comes from the signature, not an attacker-replaceable sidecar.
- Durable state uses exclusive writer locking and synced temporary-file rename;
  Unix additionally syncs the parent directory. Invalid/unwritable state fails
  closed. A process killed while holding the lock can leave a stale lock:
  after confirming no updater is running, remove **only** `state.lock`, not
  `state.json`. Keep backups of state before manual recovery.

Existing ZIP traversal/link/case-collision/size protections, cancellation,
startup acknowledgement, backup restoration and profile rollback are retained.
Filesystem rollback can restore the previous working binary without lowering
the metadata high-water marks. Local fixtures remain opt-in loopback/same-origin
and do not become production trust.

This is not protection against an attacker who can replace the executable,
modify/delete its trust state or alter the system clock persistently. Initial
install authenticity still requires a trusted installer/distribution channel.
Expiry bounds network freeze attacks; it cannot ensure network availability.
Windows power-loss durability, ACL behavior and crash recovery need desktop
validation in addition to the portable state tests.

## Production activation checklist

Do **not** activate simply to remove an error message.

1. Resolve/accept the explicit native release blockers in `SECURITY.md`.
2. Have the owner supply PUBLIC root/release keys and independently verify
   fingerprints and custody. Commit only public material to `trust-root.json`;
   pin thresholds, root version and expiry in reviewed source.
3. Configure a consistent owner-controlled repository, not Aurora's:
   - `AMANTE_RELEASE_PAGE=https://github.com/OWNER/REPO/releases`
   - `AMANTE_RELEASE_DOWNLOADS=https://github.com/OWNER/REPO/releases/download`
   - `AMANTE_RELEASES_API=https://github.com/OWNER/REPO/releases/latest/download/amante-updates.json`
4. Publish a valid signed snapshot and exercise wrong-key, expired, replayed,
   rotated, altered-archive and failed-start cases on disposable Windows installs.
   Test first install and returning-client rotation using the same static bundle.
5. Obtain independent signing/trust review and Windows desktop evidence, then
   change the explicit production flag in a reviewed release build. Update the
   intentionally-disabled configuration assertions as part of that review.

No private signing keys or real grid credentials are needed for repository tests.
Tests generate ephemeral signing keys in memory and discard them.
