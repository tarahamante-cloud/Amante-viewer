# Amante Viewer Alpha v0.9.2

## Official source

Full Aurora v0.9.2 release:
https://github.com/Aurora-Viewer/Aurora-Viewer/releases/tag/v0.9.2

Official commit: `fde838f1234ebfa2848445965f1f90541f0652f6`.

Includes chat-history discovery/import, borderless fullscreen and Alt+Enter,
motion-vector TAA, GTAO/picking improvements, hair/plant geometry fixes,
texture eyedropper/local texture workflows, build face highlighting and
link/face numbers, keyframed motion, OBS overlay protections, GPU-busy handling
and recovery after Photo quality crashes. This is the complete tagged release,
not a selection of release-note patches.

Amante branding, crest, palette, service ownership, existing profiles and
credential namespaces, maintained dependencies, and local-plugin protections
are preserved. Do not reset existing profiles for this incremental upgrade.
Protected renderer, assets, network, world and wearable source must remain
byte-identical to the official tag.

## Update notice and delivery

The native release build checks Amante's public release feed at startup.
The default Ask mode opens an offer for a newer version; Disabled mode and an
explicitly skipped tag remain respected. Version v0.9.2 is published as a normal
GitHub release with Alpha wording, plus both updater-compatible and family ZIP
names and their matching SHA-256 files. This lets existing v0.9.1 installations
find it without shipping a replacement checker first.

Add a regression asserting the login offer opens for 0.9.2 from 0.9.1 while
respecting user notice preferences. Run the complete existing offline CI and
Windows packaging gate on the exact tagged Amante source before publishing.
Never use live grid credentials in these tests.

Keep the previous download pinned until the new asset and checksum are publicly
accessible without sign-in. Then update all landing-page buttons and its
per-version announcement together through src/data/downloads.json.
Publishing the Replit website remains a separate user action.

Compilation, offline tests and ZIP verification do not demonstrate a logged-in
Windows GPU, avatar, clothing, texture, chat-history import or media session.
