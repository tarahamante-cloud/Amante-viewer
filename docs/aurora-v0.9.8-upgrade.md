# Amante Alpha v0.9.10 / Aurora v0.9.8

Integrates official Aurora v0.9.8, commit
e71e818f136a79219b675a55ea0e4a0ad9996409, including all v0.9.7 changes.
Amante's release number advances independently of Aurora's.

Includes the reorganized building tools, script recovery, Destinations,
expanded poser and appearance XML exports, AMD foliage/hair fixes,
avatar complexity notice defaults, worn-face selection, chat channel
selector, blended lashes, invisible HUD buttons, voice connection fixes,
startup splash and Ctrl-hover poser highlighting.

Preserves Amante branding, crest, settings and credential namespaces,
release feed, desktop shortcut, maintained dependencies and local-peer
protections. Protected rendering, asset, networking, world and wearable
source matches the official release exactly.

The startup splash uses the Amante name and the existing crest and palette.
The upstream one-time cache invalidation is retained; settings are kept.
The first region after updating may load more slowly while the cache rebuilds.

Release gate: complete hosted native checks and Windows packaging must pass,
then verify public assets before changing the landing-page download.
Build success does not establish logged-in GPU, outfit or media behavior.
The existing family download stays available until these gates pass.
