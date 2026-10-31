# Service ownership and content audit

## Owner-authorized fork and pending services

The existing `tarahamante-cloud/Amante-viewer` GitHub repository is accessible
but GitHub reports `fork: false` and no upstream parent. It must not be described
as a GitHub-hosted fork of Aurora. The owner authorized a new public fork at
`tarahamante-cloud/Amante-viewer-native`. GitHub confirms `fork: true` and parent
`Aurora-Viewer/Aurora-Viewer`. The existing repository was not deleted, renamed
or replaced. Native development source is on `amante-branding`, rooted like
upstream (Cargo workspace at the repository root), based on the recorded
immutable baseline. No release tags or production releases were created.

The new fork's issue tracker is enabled and its browser/API destinations are
the default public build settings. The owner created a GitHub App and supplied
its public Client ID, now the default for `AMANTE_GITHUB_CLIENT_ID`.
Device Flow, issue permissions and installation must be configured as described
in [CONFIGURATION.md](CONFIGURATION.md); no live authorization or issue
submission has been performed as part of offline verification. A connected Replit
GitHub account is agent access, **not** an OAuth app for viewer users.
No client secret is needed in the viewer.

Media distribution still needs an owner-controlled GitHub release-download
prefix and publication of the exact pinned packages. No source URL or package
has been invented. Pending configuration is visible in Preferences › Media;
manual selection of an existing compatible plugin folder remains available.

## Active and optional services

| Service / identifiers | Source | Ownership and current state |
|---|---|---|
| Source, help and release-notification links | `crates/aurora-viewer/src/links.rs`, `crates/aurora-updater/src/feed.rs` | Source/help use the authorized native fork. Notification-only discovery and About release links retain the existing Amante distribution repository; no packages moved. Notification discovery does not authorize executable replacement. |
| Issue browser/API destination | `crates/aurora-version/src/amante_config.rs` | `AMANTE_ISSUES_URL` and `AMANTE_ISSUES_API` default to the new fork's enabled issue tracker. Public build overrides must match; upstream rejected. Local report save/copy retained. |
| Viewer GitHub Device Flow identity | Same config; `bug_report/github.rs`, `crash/github.rs` | `AMANTE_GITHUB_CLIENT_ID`, defaults to the owner's public Amante App Client ID; public build override retained. GitHub's `/login/device/code`, `/login/device`, `/login/oauth/access_token` remain GitHub endpoints. Never substitute an upstream OAuth ID. |
| Executable release feed/page/downloads | Same config | `AMANTE_RELEASES_API`, `AMANTE_RELEASE_PAGE`, `AMANTE_RELEASE_DOWNLOADS`; pending. `PRODUCTION_UPDATES_READY` remains false. Authenticated update work is separate. |
| Native media package downloads | Same config; `aurora-updater/src/media_pack.rs` | `AMANTE_MEDIA_RELEASE_DOWNLOADS`, pending. Independent of viewer releases. Requires HTTPS GitHub owner/repository prefix; upstream repository rejected. Archive version, size and SHA-256 remain pinned in source. Existing archive/manifest names are compatibility identifiers, not ownership. |
| News | `crates/aurora-viewer/src/ui/news.rs`, `links.rs`, `news/amante/` | Built-in Amante news in English, French and Japanese. Remote `NEWS_FEED` is empty; no upstream feed fallback. Original upstream news is retained as source history, not bundled as Amante news. |
| Website/help/Discord/privacy | `links.rs` | Source and help point to the authorized native fork. Discord and privacy URL empty pending owner configuration. Privacy help explicitly says pending in all locales. |
| Second Life/OpenSim grids, login/caps, voice, marketplace, currency and maps | `aurora-net/src/{login,currency,voice}.rs`, viewer grid/session and marketplace modules | Grid infrastructure and negotiated capabilities, not viewer-owned reporting. Retained for compatibility; custom grid endpoints remain the user's choice. No live-grid validation performed. |
| GIF search | `crates/aurora-viewer/src/gif_search.rs`, `ui/options/pages/chat.rs` | GIPHY and KLIPY provider endpoints. User-configured provider keys; no Amante-owned provider account asserted. Preserve provider names, attribution and dashboard links. |
| Link previews and embedded web/media | `link_preview/`, `media/` | Provider URLs, grid media and user-selected content, subject to existing filters/consent. Do not rewrite user URLs into Amante URLs. |
| Developer dependencies and plugin build inputs | Cargo manifests, native plugin CMake/scripts and retained workflows | Upstream source/dependency references and third-party build downloads are not Amante support services. Preserve technical provenance. Retained upstream publishing workflows must be reviewed before owner execution. |

Regenerate `service-references.json` from the workspace root with
`node scripts/capture-amante-service-references.mjs`. It includes added files,
not just the original baseline. It is a literal-reference inventory, not a
claim that all listed URLs are fetched or that third-party services were tested.
Dynamic grid capabilities, user-entered URLs, GitHub redirects and public
build variables cannot be exhaustively represented by literal matching.

## Multilingual content decisions

English, French and Japanese catalogs now explicitly identify the running app
as Amante in About/help, menus, exit/crash dialogs, graphics failures,
preferences, privacy notices, monitoring and update/report prose.
The rendering-time allowlist in `aurora-i18n/src/branding.rs` covers remaining
product-only CLI/skin/AO/status messages and instructions that must match the
already-branded inventory roots, bridge and Blender add-on. It does not perform
a catalog-wide replacement or modify any user's content.

The following Aurora references are intentional, not unfinished renames:

- Original demo worlds, residents, regions, sample notecards, photos and chat.
- Upstream staff badges and legal authorship/license notices in About.
- Existing residents' inventory/object names, whether they contain Aurora or
  any other name. Do not migrate or rename residents' content. Current tool
  instructions use `#Amante`, `Amante AO`, `Amante Library` and `Amante Bridge`
  because these are the actual names created by this fork.
- `Aurora 3D`, Aurora pose formats, named photo presets and bundled asset names.
- Accurate third-party add-on names and technical references. This fork's
  bundled Blender add-on is already named `Amante LiveMesh`, with Amante
  sidebar/connect labels; instructions match that implementation.
- Internal crate/message IDs, protocol/schema names and `AURORA_*` test switches.
- Technical citations and offline test fixtures, including rejected upstream URLs.

Automated regressions distinguish preserved names from Amante product copy.
They do not certify desktop operation or a released Windows binary.
