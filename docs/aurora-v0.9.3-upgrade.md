# Amante Alpha v0.9.7 — official Aurora v0.9.3

Official baseline: `7851d13e61b3f83c7a8ff367f7678fbfd2e3c156`.
This is a complete upstream release merge, including AMD FSR 3.1,
GPU-reset recovery, shared-memory texture budgets, alpha-aware picking,
terrain and photo fixes, asynchronous logging, voice stability, building,
inventory, camera and chat improvements, periodic update notices,
media-download consent and removal of external registrations.

Amante keeps its crest, palette, ownership, credential/profile namespaces,
maintained dependency fixes and local-peer protections. Protected renderer,
asset, network, world and wearable engine files match the official release.
Settings and saved accounts remain in the same Amante Alpha profile.
The upstream cache migration clears cached assets once, not settings:
the first region can load more slowly.

The Windows viewer creates an Amante Viewer Alpha desktop shortcut on its
first normal launch from the extracted package. No administrator access is
needed. Keep that extracted folder in place. The operation runs off the frame
thread, does not replace a user's unrelated shortcut, and records successful
creation so deleting the shortcut does not recreate it at each launch.
The viewer's removal feature removes its own matching shortcut.
CI exercises the real Windows shortcut API inside a temporary directory.

The Amante release number advances to v0.9.7 because v0.9.3 was already used
for the previous Aurora v0.9.2-based release and v0.9.4 through v0.9.6 were source-only
candidates. The login-notice regression uses the upstream generation-aware
event helper. The maintained font adapter also handles the new floating-text
baseline metrics; no legacy parser is reintroduced. Existing tags are not rewritten.
Publishing waits for the complete source CI, extracted executable, package
and checksum verification. The release provides a family-named ZIP and a
versioned ZIP for the native updater. Login update notices respect disabled
updates and skipped versions; official periodic checking is retained.

Only after anonymous download and checksum verification does the landing page
switch every download button and version-specific popup to this release.
Publishing the Replit website is a separate user action.
Compilation and offline tests do not establish logged-in Windows GPU or
Second Life runtime coverage.
