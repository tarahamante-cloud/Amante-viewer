# Amante Viewer

Amante Alpha is a native Windows viewer for Second Life, rebuilt from a clean
Aurora Viewer snapshot with the Amante name, family crest and green/copper palette.

- [Windows downloads and checksums](https://github.com/tarahamante-cloud/Amante-viewer/releases)
- [Clean rebuild and configuration](amante-viewer/AMANTE.md)
- [Complete upstream/branding source inventory](amante-viewer/amante-upstream-manifest.json)
- [Upstream documentation](amante-viewer/README.md)

These are unsigned development builds, not a security-certified release.
Download the actual versioned ZIP and verify its published SHA-256. New source
changes do not change an already downloaded executable; install the newly
built ZIP to get changes. Offline tests do not verify live Second Life rendering.
Previous Amante-only texture, dependency, clothing, browser, toolbar and updater
patches have been discarded; those systems now use upstream code.

## Credits and compatibility

Amante is based on Aurora Viewer and retains its GPL-3.0-or-later license,
authors' notices and original technical history. The source directory retains
internal `aurora-*` crate/module names and compatibility switches. These are
not the product name shown in the interface.

See [LICENSE](amante-viewer/LICENSE), [NOTICE](amante-viewer/NOTICE.md), and
[the immutable baseline](amante-viewer/AMANTE.md).
