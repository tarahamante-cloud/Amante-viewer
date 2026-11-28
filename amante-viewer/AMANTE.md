# Amante Viewer Alpha v0.9.16

Based on official Aurora Viewer release `v0.9.13`, commit
`e961b7df61c49552773f3bc209d7d80d92a0b16b`, including the cumulative
`v0.9.12` update.

This update includes Aurora's new interface languages, Game Controls, world
photo looks and LUTs, Phototools lighting and depth-of-field controls,
ray-traced shadows and indirect lighting, optional planet curvature, and
updates across chat, avatars, inventory and building tools. The rendering,
texture, clothing, networking and other protected runtime files are
byte-identical to official Aurora `v0.9.13`.
Amante retains its name, crest, palette, profile and credential namespaces,
owner feeds, packaging, maintained dependencies and local-peer protections.
Amante's mutual Blender session pairing remains in place; use the matching
Amante Blender add-on.

This Alpha also keeps Amante's reviewed dependency security fixes:
maintained font metadata parsing and Linux credential/D-Bus libraries,
patched certificate validation, Markdown parsing and constant-time utilities.
See `amante-security-manifest.json` for the exact security-maintenance
exceptions. Those dependency changes do not locally rewrite Aurora's
rendering, world-texture, networking or wearable engine code.

LICENSE and NOTICE.md retain Aurora and third-party attribution. The source manifest records every original and branded file hash.

This incremental update keeps existing Amante settings and saved accounts.
The cache may take longer to rebuild the first region after updating. A
Windows desktop shortcut is created on the first normal
launch from the extracted folder, without administrator access. Keep that
folder in place. Removing the viewer's registrations also removes its own
shortcut, never another program's shortcut.
Reports and updates use the Amante repository. Direct OAuth reporting is
unconfigured rather than using Aurora's app. Existing user skins remain user
choices. Native build/test checks do not prove texture loading in a live
Second Life session.
