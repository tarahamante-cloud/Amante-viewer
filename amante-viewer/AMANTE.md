# Amante Viewer Alpha v0.9.2

Based on official Aurora Viewer release `v0.9.2`, commit
`fde838f1234ebfa2848445965f1f90541f0652f6`.

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

This clean Alpha uses its own fresh profile and cache, so it does not import settings or cached textures from earlier Amante builds. Add your account again on first launch. No old credentials, profiles or user caches are deleted. Reports and updates use the Amante repository. Direct OAuth reporting is unconfigured rather than using Aurora's app. Existing user skins remain user choices. Native build/test checks do not prove texture loading in a live Second Life session.
