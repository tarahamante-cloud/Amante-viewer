# Amante Viewer Alpha v0.9.1

## Source and scope

Official upstream release: https://github.com/Aurora-Viewer/Aurora-Viewer/releases/tag/v0.9.1

Upstream commit: `76f7cd150bb6238fbe5eb895e47ee84777896dfe`.

This is the full official release update, including rendering and texture
changes, not the earlier renderer-frozen feature experiment. Amante identity,
crest, palette, owner feeds, account namespaces and existing dependency/local-peer
protections are retained. The complete upstream Quick Preferences, browser,
Friends Only and toolbar implementations replace the unfinished local substitutes.

The source inventory verifies every official file. Renderer, asset/texture,
network, world and wearable engine files must be byte-identical to the tagged
upstream release. Do not loosen that requirement to fix a build failure.

## Dependency compatibility

The new font-picker previews use the maintained fontations adapter rather than
reintroducing ttf-parser. A CFF1 font whose letter names cannot be validated is
previewed in the interface typeface; selecting and using the font still works.
Security exceptions remain explicitly hashed and allowlisted.

## Delivery gate

Build and test the exact tagged source in the Windows and Linux workflow.
Publish only its verified ZIP and matching SHA-256 checksums. Keep the existing
working family link until the new public asset actually exists and is accessible
without signing in. Then pin every landing-page button and the release popup to
that one asset through `artifacts/amante-downloads/src/data/downloads.json`.
Set its `announce` flag only when the release is downloadable.

The native updater continues to use Amante's public GitHub feed, not Aurora's.
The public release needs a numerically higher semantic version and the compatible
Windows asset/checksum names produced by the publisher. Preserve Alpha wording
in the release title and landing page.

Compilation, offline tests and package inspection are separate from actual
Windows GPU/Second Life validation. Do not describe them as proof that a
logged-in avatar, clothing, environment, teleport or media session has been tested.
