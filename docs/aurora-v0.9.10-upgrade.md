# Amante Alpha v0.9.12 / Aurora v0.9.10

Integrates complete official Aurora v0.9.10, commit
1c9c39c2c33645fdbcf4351245bc15ee72d8dde9, on top of the complete v0.9.9 merge.
Amante release numbers advance independently of Aurora.

Includes Director camera timelines and recording (requires ffmpeg),
transparent-surface ordering, expanded weather, Collada/OBJ export,
1024 m draw distance, graphics and mirror fixes, Conversation Log,
contact sets and radar markers, avatar, building, inventory and sound
improvements, and the upstream updater recovery fixes.

Preserves Amante branding, crest, settings and credential namespaces,
release feed, desktop shortcut, maintained dependencies and local-peer
protections. Rendering, asset, network, world and wearable engine source
must match the official release exactly.

The official one-time cache invalidation is retained; settings are kept.
The first region may load more slowly while the cache rebuilds.

Release gate: full hosted native checks and Windows packaging must pass,
followed by public asset checksum verification before replacing website links.
The existing family download stays available until those gates pass.
Automated checks do not establish logged-in desktop/GPU behavior.
