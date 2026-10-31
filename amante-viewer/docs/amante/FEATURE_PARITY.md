# Source and feature parity

## Baseline captured before viewer edits

Upstream: `Aurora-Viewer/Aurora-Viewer`, branch `main`, commit
`d555fb677004bee8fd9e7f12b0e33a8978e77d4c`.

The original checkout has 8,643 tracked files and 1,463 Rust source files.
There are no declared submodules or LFS-managed files at this baseline.
The full tree, its assets, native dependencies, platform files and licenses
are retained, rather than replaced by a browser shell.

`upstream-inventory.json` was generated from immutable upstream Git objects
before application edits. It records every original file's Git blob identity,
495 UI modules, 785 static menu registration call sites, 516 public settings
declarations, 200 keybinding/toolbar declarations and all three locale catalogs.

These are **source counts**, not counts of unique features or proof of runtime
parity. Dynamic lists, platform-gated actions and nested settings require human
inspection. The CSV is an item-by-item source table derived from this inventory:

- [Full parity table](feature-parity.csv)
- [Immutable baseline inventory](upstream-inventory.json)
- Run `node scripts/verify-viewer-preservation.mjs` from the workspace root to
  check every original file and regenerate the table.
- The detailed current blob comparison is written to the ignored
  `generated-artifacts/source-preservation.json`.
- Comparisons use Git's canonical text/binary handling. Windows checkout
  line-ending conversions are reported separately, not as source or asset
  edits; original raw-byte identities are also retained in the detailed report.
- Use `--require-tracked` to reject baseline files that exist locally but
  would be omitted from a Git export.
- Package-backed attribution requires the exact built Git revision and a clean
  native source checkout (apart from the generated CSV). An unrelated package
  report cannot certify the current source tree. Regression checks:
  `node --test scripts/verify-viewer-preservation.test.mjs`.

## Feature groups found in the actual source

Every row below means **implementation retained; desktop behavior unverified**.
The baseline Linux compile succeeded. Neither source presence nor compilation
means that a live-grid feature has been tested.

| Area | Actual implementation locations | Verification / limitations |
|---|---|---|
| Menus, toolbar and shortcuts | `src/ui/bars`, `src/ui/toolbar`, keybind declarations in inventory | No layout redesign; static registration table available |
| Login, grid sessions, MFA and credentials | `aurora-net/src/login.rs`, viewer credentials/session modules | HTTPS and same-origin redirect restrictions added; grid compatibility untested |
| Rendering, terrain, lighting and materials | `aurora-render`, `aurora-prim`, viewer world modules | Original native Vulkan implementation retained; no graphics runtime claim |
| Avatars, outfits and appearance | `src/ui/appearance`, `src/ui/ao`, `src/ui/gestures` | Original data models, UI and animation tools retained |
| Animation and posing tools | `src/ui/animation_explorer`, viewer animation/poser modules | Interactive desktop exercise pending |
| Inventory, asset editing and scripts | `src/ui/inventory`, `src/ui/scripts`, `aurora-assets` | Original editors retained; Markdown dependency patched |
| Chat, IM, contacts and groups | `src/ui/chat`, `src/ui/contacts`, `src/ui/group_profile`, `aurora-net` | Sending/receiving on a real grid not tested |
| WebRTC voice and audio | `aurora-voice`, `aurora-audio` | Native libraries retained; device and call tests pending |
| Search, places, people, maps and portals | `src/ui/search`, `places`, `people`, `worldmap`, `portals` | Source preserved, no live service verification |
| Land, estate, region and build controls | `src/ui/land`, `src/ui/region`, `aurora-net` | Permissions and server-side outcomes need human verification |
| Photos, camera, viewfinder and capture | `src/ui/snap`, `src/ui/viewfinder`, camera modules | No desktop screenshots or export-flow certification yet |
| Media plugins, textures and sound exploration | media/native plugin modules, `texture_library`, `texture_picker`, `sound_explorer` | Manual plugin selection retained; automatic Amante plugin distribution pending |
| Marketplace, money and commerce | `src/ui/marketplace_listings`, `money_tracker`, `aurora-net` | No purchases, payments, inventory writes or live transactions performed |
| Preferences, notifications and monitoring | `src/ui/options`, `notifications`, `notification_center`, `monitor` | Amante data/keyring namespaces isolated; Windows registration pending CI/runtime |
| Blender live integration | viewer `blender_live`, bundled addon source | Configuration path isolated; Blender integration not exercised |
| About, news and diagnostics | `src/ui/about`, `news`, bug-report and crash modules | Local reports retained; browser reports use the authorized fork; direct sending awaits owner OAuth |
| Windows updater, safe extraction and rollback | `aurora-updater`, viewer updater UI | Implementation and local fixtures retained; production deliberately inactive |

Paths in this table beginning with `src` are under `crates/aurora-viewer`.
Internal `aurora-*` names, compatibility protocols and original asset names
are not themselves evidence that an action contacts Aurora.

## Changes that affect behavior

1. TLS endpoints must present a certificate matching the requested hostname.
   Linden's CA remains trusted, but a CA-signed mismatched certificate is rejected.
2. Login requires HTTPS. Credential redirects must stay on the selected login
   origin, including the port. Cross-origin and plaintext custom-grid login
   flows fail explicitly instead of forwarding secrets.
3. Amante has independent settings, cache/log and keyring namespaces. It does
   not silently import Aurora credentials or automatically migrate its data.
4. Reports no longer default to Aurora's repository or OAuth application.
   Browser drafts use the authorized fork; local saving/copying remains
   available while direct-send OAuth configuration is pending.
5. Production updater requests are blocked pending authenticated Amante release
   metadata. Existing checksum, extraction and rollback implementations remain,
   but are not a substitute for signed update authorization.
6. Primary titles, login channel, user agent, Windows resource identity,
   notifications and distributed executable identify Amante. The comprehensive
   multilingual branding/content audit is documented in
   [SERVICE_OWNERSHIP.md](SERVICE_OWNERSHIP.md). Product prose uses Amante;
   upstream demo-world names, named formats/presets, credits and accurate
   technical references intentionally retain Aurora. Tool instructions match
   the fork's existing Amante inventory roots, bridge and Blender labels;
   residents' existing content is not renamed.
7. Native media distribution has a separate owner-controlled build setting.
   Until configured, Preferences shows it as pending/unavailable rather than
   contacting Aurora or repeatedly failing an update request.

## Evidence levels

Track source retention, successful compilation, successful packaging, offline
regressions and desktop/runtime outcomes separately. Do not mark a feature
“tested” merely because it appears in this table or builds.
