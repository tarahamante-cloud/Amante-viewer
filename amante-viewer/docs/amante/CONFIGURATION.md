# Independent Amante services

There are no owner credentials, signing keys or working release/reporting
destinations in this checkout. Public build settings are optional; missing
configuration is explicit and never falls back to Aurora.

## Public reporting settings

The following public variables are read at compile time in
`aurora-version/src/amante_config.rs`:

| Variable | Required value / purpose |
|---|---|
| `AMANTE_ISSUES_URL` | Owner-controlled GitHub repository issues page |
| `AMANTE_ISSUES_API` | Matching `https://api.github.com/repos/OWNER/REPOSITORY/issues` |
| `AMANTE_GITHUB_CLIENT_ID` | Public identifier of Amante's own GitHub App with Device Flow enabled |

Repository URLs must describe the same destination. Aurora's upstream
repository is explicitly rejected. Browser reporting needs the two repository
settings; direct sending additionally needs the OAuth client ID.

OAuth tokens remain in memory or Amante's separate OS credential service,
according to the retained reporting implementation. Never put a client secret,
access token, refresh token or signing key in source, public variables or logs.
GitHub's device authorization endpoints remain GitHub endpoints; they are not
an Aurora-owned service. No real authorization was performed in automated tests.

## Update and plugin distribution settings

- `AMANTE_RELEASES_API`
- `AMANTE_RELEASE_PAGE`
- `AMANTE_RELEASE_DOWNLOADS`

All default to empty. **They do not activate production updates.**
`PRODUCTION_UPDATES_READY` remains false because authenticated release metadata,
the Amante public trust root, key rotation, anti-rollback policy and owner
release authorization still need implementation and review.

Do not flip this guard just to make update checks work. Checksums delivered
beside an archive do not establish publisher authenticity. CI packages are
downloadable development builds, not automatically trusted update releases.

The existing loopback-only updater fixtures remain available without a real
grid, repository token, production key or published release. Automatic native
media-plugin distribution also needs an Amante-controlled package source;
manual plugin-folder selection and all plugin implementations are retained.

## Data isolation

- Project directories: qualifier `org`, organization `Amante`, app `AmanteViewer`.
- OS credential services: `Amante Viewer`, `Amante Viewer MFA`,
  `Amante Viewer GitHub`.
- Windows notification identity: `Amante.Viewer`.
- SLURL handoff named pipe: `amante-viewer-slurl-…`.
- Distributed executable: `amante-viewer.exe` / `amante-viewer`.

Legacy protocol/environment names such as `AURORA_DEMO` and the internal
installation-manifest schema remain for source compatibility. They do not
select Aurora's settings directory or authorize an upstream update.

## Retained upstream references

Upstream authorship, licenses, technical compatibility names, original
documentation and read-only help/news links remain in the source. These must
not be mistaken for Amante service ownership. A full external-link/content
audit, Amante website/help/news destinations and the remaining visible branding
review are pending. Do not invent owner URLs or describe them as configured.
