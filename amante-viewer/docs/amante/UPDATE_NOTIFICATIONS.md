# Amante startup update notices

Release builds, including development Windows downloads, check for newer
Amante releases once when opened. The check runs in the background and times
out after eight seconds. It never blocks login. Offline/network failures do
not open an unsolicited error pop-up; a manual check can show the error.

The existing update preference still applies: Disabled suppresses unsolicited
notices; Ask and Automatic both require a click in notification-only mode.

When a newer version is available, the existing update dialog shows release
notes, the installed/new version numbers, and **Download** / **Later**:

- Download opens that version's official GitHub release page in the browser.
  The user chooses and downloads the Windows ZIP there.
- Later dismisses the notice until the next launch. Closing the dialog does
  the same thing.
- Nothing is silently downloaded, installed, restarted, reset or deleted.

## Publishing versions the viewer can discover

The notification source is the public `tarahamante-cloud/Amante-viewer`
GitHub Releases API. GitHub Actions artifacts alone are **not** releases.
There were no published releases when this feature was added, so the check
correctly offers nothing until a release with a Windows package is published.

1. Build from a proper version tag such as `v0.1.0`, with the same version
   embedded in the binary. Version numbers must increase for later releases.
2. Publish a non-draft GitHub release with that tag and attach the complete
   packaged Windows ZIP named `Amante-Viewer-v0.1.0-windows-x64.zip`.
   Use the real version in the filename. Do not rename a development ZIP to
   pretend it contains a newer version.
3. Publish its SHA256 sidecar too, using the existing packaging workflow.
   Notifications require a nonempty Windows ZIP; they do not download it or
   interpret executable update policies.

Stable users are offered only stable releases. Beta users can also receive
`-beta` releases. Drafts, malformed versions, Git-distance builds, releases
without a Windows package, and same/older versions are ignored. Selection uses
semantic version precedence, not upload date. Up to the latest 100 releases
are considered.

Links are constructed from the fixed Amante repository and a validated version
tag, not an arbitrary URL supplied by release metadata. Requests are HTTPS,
bounded and time-limited.

The authenticated automatic replacement path remains disabled. Notifications
are not evidence that an executable is signed or verified, and do not change
Chrome's download decisions. Disposable offline updater fixtures retain their
existing installation tests and behavior.

Existing downloaded binaries cannot acquire this feature themselves. Users
must download a newly built viewer containing it once.
