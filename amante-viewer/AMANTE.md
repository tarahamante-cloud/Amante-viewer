# Amante Viewer Alpha v0.9.8

Based on official Aurora Viewer release `v0.9.6`, commit
`019d41a5ba4b87fb2f6e7072ae5c003b31017c26`.

This update includes the full official release, including its rendering,
texture, clothing, networking and performance changes. Those engine files
are byte-identical to the official release, not locally rewritten.
Amante retains its name, crest, palette, profile and credential namespaces,
owner feeds, packaging, maintained dependencies and local-peer protections.
Aurora's complete Quick Preferences, toolbar, browser and Friends Only
implementations supersede the earlier unfinished local UI substitutes.

This Alpha adds reviewed dependency security fixes to that clean baseline:
maintained font metadata parsing and Linux credential/D-Bus libraries, patched
certificate validation, Markdown parsing and constant-time utilities.
See the included `amante-security-manifest.json`
for the exact security-maintenance exceptions. Rendering, world textures,
networking and wearable engine code remain unchanged.

LICENSE and NOTICE.md retain Aurora and third-party attribution. The source manifest records every original and branded file hash.

This incremental update keeps the existing Amante Alpha settings and accounts.
The official cache format upgrade clears the cache once; the first region can
load more slowly. A Windows desktop shortcut is created on the first normal
launch from the extracted folder, without administrator access. Keep that
folder in place. Removing the viewer's registrations also removes its own
shortcut, never another program's shortcut.
Reports and updates use the Amante repository. Direct OAuth reporting is
unconfigured rather than using Aurora's app. Existing user skins remain user
choices. Native build/test checks do not prove texture loading in a live
Second Life session.
