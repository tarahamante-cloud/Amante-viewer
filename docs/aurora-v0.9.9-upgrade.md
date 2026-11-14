# Amante Alpha v0.9.11 / Aurora v0.9.9

Integrates the complete official Aurora v0.9.9 release, commit
b630e7391e4454f1d9aec1a2e5b6521f6828858e, on top of the previous v0.9.8 merge.
Amante release numbers advance independently of Aurora.

Includes Weather, SLua editing, Avatar Studio and animation editing,
gamepad/VR controls, selection beams, lighting and mesh-head fixes,
camera tracking, interface opacity, nearby media controls, web clipboard
and scripted windows, group moderation and contact improvements,
building and COLLADA improvements, AO fixes and parcel music crossfades.

Preserves Amante branding, crest, settings and credential namespaces,
release feed, desktop shortcut, maintained dependencies and local-peer
protections. Rendering, asset, network, world and wearable engine source
must match the official release exactly.

The official one-time cache invalidation is retained; settings are kept.
The first region may load more slowly while the cache rebuilds.

Release gate: full hosted native checks and Windows packaging must pass,
followed by public asset checksum verification before replacing website links.
The existing family download stays available until those gates pass.
Automated checks do not establish live-grid rendering, weather, voice,
inventory, script recovery or media behavior.
