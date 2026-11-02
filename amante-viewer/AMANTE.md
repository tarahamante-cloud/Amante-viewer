# Amante Viewer Alpha — clean upstream rebuild

Based on Aurora Viewer commit `b21e5c27880176978d4a301220645c2830fef3a9`.

The runtime is upstream Aurora with Amante name, crest, palette, product directories, owner links and package identity. The previous Amante-specific renderer, texture, clothing, browser, toolbar and dependency changes are not included.

LICENSE and NOTICE.md retain Aurora and third-party attribution. The source manifest records every original and branded file hash.

This clean Alpha uses its own fresh profile and cache, so it does not import settings or cached textures from earlier Amante builds. Add your account again on first launch. No old credentials, profiles or user caches are deleted. Reports and updates use the Amante repository. Direct OAuth reporting is unconfigured rather than using Aurora's app. Existing user skins remain user choices. Native build/test checks do not prove texture loading in a live Second Life session.
