# Local media and Blender authorization

## Trust and compatibility

Media still launches the configured SLPlugin with its ordinary port argument.
Its LLSD/XML wire protocol and shared-memory messages are unchanged. Before
accepting hello or any plugin data, the host verifies the **client endpoint**
belongs to that exact launched child process. A competing same-user process,
a child of SLPlugin, or another executable at the same path is not accepted.
The launched child is held until exit/kill, preventing PID reuse during
authorization. Linux requires readable procfs information; Windows uses the
IPv4 owner-PID TCP table. Missing/unsupported OS authorization denies rather
than reverting to first-connection trust. Rejection preserves the listener
and the original 60-second launch deadline.

Blender uses an explicit private per-listener session code. See
[`../BLENDER_LIVE.md`](../BLENDER_LIVE.md) for setup and the mutual HMAC
handshake. Authentication candidates cannot read rig/joint information, use
shape/import controls, or mutate scene state. Proofs are role separated and
freshly challenged, not a reusable bearer credential sent on the socket.
The existing protocol version, snapshot frames, materials, shape and LOD
features are preserved after pairing; both viewer and addon must be upgraded.

These boundaries do not isolate code injected into a trusted process, a
debugger/admin that can read its memory or duplicate its handles, or a user who
shares the pairing code/clipboard. A trusted media plugin remains native code.
They are not a promise that a sustained OS-level TCP/backlog flood cannot
temporarily deny service. Keep runtime evidence separate from source checks.

## Media limits (per plugin host)

| Resource | Bound / result |
| --- | --- |
| XML frame | 1 MiB before NUL; malformed XML is fatal, not silently skipped |
| Partial frame | Five seconds absolute from first buffered bytes |
| Socket work per idle | 256 KiB read and 256 KiB write; at most 64 decoded messages |
| Received queue | 256 messages and 4 MiB serialized input, whichever first |
| Outgoing socket queue | 4 MiB total; individual message at most 1 MiB |
| Pre-ready/media-class send queue | 256 messages and 4 MiB |
| Unconsumed media events | 256 events (checked before each incoming media message) |
| Shared memory | Nonzero, at most 64 MiB each, four segments and 128 MiB total including pending removals |
| Texture parameters | Depth 1–4 bytes; padding -1–4096; requested dimensions clamped before arithmetic |
| Texture response | Must match the outstanding request's sent dimensions, not a newer UI resize; texture axes 1–4096 |

Exceeded protocol/queue/memory bounds fail the plugin explicitly, close sockets,
terminate/reap the child, and release pending buffers/mappings. Normal shared
memory removal still waits for the plugin's acknowledgement. Startup,
15-second running heartbeat and shutdown timeouts remain; early heartbeats
cannot extend the launch/load deadline. Fragmented valid frames are supported.
Failure recovery is a new plugin launch, not trusting a replacement connection
to an old failed instance.

## Blender limits

Authentication is nonblocking, with at most 16 pending candidates, 16 accepts
per poll, fixed-size handshake buffers and a two-second absolute deadline.
Invalid proofs/magic, EOF, expired and over-capacity candidates are closed.
An authenticated connection exclusively serves the original transport; queued
unpaired clients cannot overwrite its state. The existing 64 MiB frame bound
remains. Frame headers expire after ten seconds and bodies after five seconds,
even if a peer trickles bytes; stop interrupts reads. A dropped connection
clears per-connection pending shape/import negotiation and allows re-pairing
with the same session code. Stop/start rotates the code.

## Reproducible offline verification

From `amante-viewer/`:

```sh
cargo test --profile ci --locked -p amante-local-ipc -p aurora-media
python3 -m unittest discover -s blender/tests -p test_client.py
cargo clippy --profile ci --locked -p amante-local-ipc -p aurora-media --all-targets -- -D warnings
cargo check --profile ci --locked -p aurora-viewer --bin aurora-viewer
```

On Windows use `python` rather than `python3`. Both OS jobs in
`.github/workflows/amante-build.yml` run these synthetic suites; the Windows
tests exercise the real owner-PID API and an independently spawned Rust
synthetic plugin, not a mocked table. Linux exercises the real procfs path.
The ignored Rust test is a **subprocess fixture**, automatically executed by
the parent tests, not a skipped security scenario. Pairing tests cover
silent/wrong/legacy clients, bounded admission, expiry, replay/reflection,
stop/deadline, reconnect, code rotation and the actual Python addon talking
to the real Rust gate.

The native viewer tests additionally retain paired snapshot, shape, LOD and
capability scenarios. Real CEF/VLC/GStreamer, Blender UI, GPU and Windows desktop
execution are not established by Linux synthetic tests. No live-grid accounts,
download activation, signing policy or released packages are changed here.

### Verification observed in this Linux workspace

- 19 Rust tests passed across the local-IPC and media suites; the subprocess
  helper is marked ignored for standalone discovery and exercised by its parents.
- A real synthetic child echoes size acknowledgements over TCP; resizing while
  a previous request is outstanding accepts the old reply and then sends the
  latest requested size. Unsolicited replies remain rejected.
- 13 Python client tests passed, including fake-viewer rejection and retained
  shape/LOD/material negotiation behavior.
- Targeted all-target Clippy passed with warnings denied; changed Rust files
  passed formatting checks.
- The real native viewer binary target and its complete test target both passed
  compile checks when run separately with one build job. Test-target warnings
  were existing unused/dead-code diagnostics, not compile failures.
- A simultaneous all-target check with two jobs was killed under the workspace
  memory limit. The separate checks above subsequently completed successfully.
- The complete native test executable was not linked/run, no Windows binaries
  were executed here, and no real Blender/plugin/GPU desktop session was run.
