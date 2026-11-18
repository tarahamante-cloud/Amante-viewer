# Amante Alpha v0.9.14 / Aurora v0.9.11

Integrates official Aurora v0.9.11 at
3d1420a932af25ff62be3d22a4c5718e428b8c41, including all earlier updates.
Includes expanded Director and photo tools, 360-degree snapshots, gaming mode,
clouds and frame-rate controls, rendering fixes, pose and animation improvements,
chat filters, favorite wearables, dual-panel inventory, building tools,
proxy settings, external script editing, RLVa and world/network fixes.

Amante retains its crest, branding, settings, credential namespace, desktop
shortcut and update feed. Protected rendering, asset, networking, world and
wearable files remain byte-identical to the official release.

Security integration:
- Keep Amante's existing mutual, per-session Blender pairing rather than replace
  it with Aurora's new one-way token-file protocol. Use the Amante Blender addon.
  All unrelated upstream LiveMesh changes and UI regression tests are included.
- Apply the upstream D-Bus listener startup ordering fix using Amante's maintained
  asynchronous zbus API.
- Retain upstream update/rollback preservation of the user's personal files and
  adapt its new executable-name fixtures to Amante.
- Keep new SOCKS proxy credentials in Amante's own credential namespace.
- Accept the official rustls-webpki 0.103.17 security patch; regenerate library
  notices from the merged lockfile, including complete aws-lc-sys attribution.

Optional recording requires ffmpeg; do not imply the pack is bundled.
Full hosted builds and native tests must pass before publishing a Windows ZIP.
Verify the public package checksum before replacing website download links.
Automated checks do not establish logged-in desktop/GPU behavior.
