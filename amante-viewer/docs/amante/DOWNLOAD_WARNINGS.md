# Investigating a blocked Windows download

Do not assume Chrome's "Dangerous download blocked" warning is caused only by
an unsigned or unfamiliar executable. Build and unit-test success are not
malware-scanning results. Do not disable browser protection, add antivirus
exclusions, rename/repack the package to evade detection, or run a blocked file.

## Scan the existing package on a Windows CI runner

The separate **Investigate an Amante Windows download** workflow scans an
existing artifact rather than rebuilding it. First commit and push both
`.github/workflows/amante-download-scan.yml` and
`scripts/scan-amante-download.ps1` into the GitHub repository. A manually
dispatched workflow must exist on the repository's default branch to appear
under Actions.

1. Open **Actions → Investigate an Amante Windows download → Run workflow**.
2. Enter the successful build run's numeric ID: the final number in its
   `https://github.com/OWNER/REPOSITORY/actions/runs/NUMBER` URL.
3. Run the workflow. It downloads `Amante-Viewer-windows-x64` from that run,
   checks the versioned ZIP against its checksum sidecar, updates Defender
   signatures, and scans the archive without launching its contents.
4. Inspect the scan step and `Amante-download-scan-evidence` artifact. A failed
   scan or unavailable scanner is a failure, not a clean result.

This workflow does not publish a release, sign code, change the original ZIP,
or claim to remove Chrome's warning. Its token has only repository-content
and Actions read permissions. Never supply a GitHub token in chat.

If Defender identifies a threat, preserve the detection name, package SHA-256,
build revision and scan output, and investigate before distributing anything.
If it reports no detection, that is only one scanner's result—not proof of
safety or a diagnosis of Google's block. Further review of the exact package,
its dependencies, download source and Google's finding is still required.
Code signing establishes publisher identity but is not malware clearance.

## References

- [Google Chrome download warnings](https://support.google.com/chrome/answer/6261569?hl=en)
- [Google's guidance for software publishers](https://support.google.com/webmasters/answer/3258249)
- [Microsoft Defender command-line options](https://learn.microsoft.com/en-us/defender-endpoint/command-line-arguments-microsoft-defender-antivirus)

Microsoft documents that `-Scan -ScanType 3 -DisableRemediation` scans archives,
ignores exclusions, and reports detections in command output without applying
remediation. A nonzero exit code may mean either a detection or a scanning
error; preserve the output rather than labelling every failure as malware.
