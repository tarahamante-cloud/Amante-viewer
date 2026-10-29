# Human desktop validation checklist

All items are **pending** until a person records results on an actual desktop.
Source presence, a successful compile, or a downloaded ZIP does not check a box.

Record build revision, baseline commit, package SHA-256, OS, GPU/driver,
display scaling, selected language and whether the run is offline or live.
Use a clean Amante profile first. Confirm Aurora data and keyring entries are
not read, overwritten or deleted.

## Offline first

- [ ] Packaged `amante-viewer.exe --version` reports Amante and exits without
      starting a window, network login or credential prompt.
- [ ] `AURORA_DEMO=1` starts the native renderer, not a web page.
- [ ] Window, login, About and Windows file/notification identities show Amante;
      record remaining Aurora product prose separately from retained credits,
      presets, demo objects and protocol names.
- [ ] Compare every menu, toolbar entry, shortcut, context menu and Preferences
      page against the baseline and `feature-parity.csv`.
- [ ] Exercise keyboard-only operation, focus, dialogs, resizing, UI scaling,
      theme switching and en/fr/ja layouts.
- [ ] Exercise camera, photo settings, capture/export, build/selection tools,
      avatar appearance, AO, gestures, animation exploration and posing.
- [ ] Open inventory editors, scripts, Markdown/notecards, search/map/land/region
      panels, contacts, groups, marketplace and media tools in their available
      offline states. Distinguish demo fixtures from live data.
- [ ] Select a local media-plugin folder; verify decoder/plugin behavior without
      claiming automatic Amante plugin distribution is configured.
- [ ] Exercise local bug/crash report preview, redaction, copy and save.
      Unconfigured remote actions must give an explicit pending message and
      must not contact Aurora.
- [ ] Verify log files are in Amante's directory and contain no login secrets,
      MFA values, OAuth tokens, capability URLs or private report bodies.
- [ ] Test the loopback-only update fixture, cancellation, corrupt archives,
      extraction rejection, rollback and Amante executable replacement.
      Never use a production signing key or a published release for this.
- [ ] Confirm a configured feed URL alone cannot enable production updates.

## Human-only live-grid validation

Never put actual credentials into automated scripts or test-agent instructions.
Only the account owner may choose to do these tests interactively.

- [ ] Login, logout, MFA and trusted same-origin redirects work on each supported
      grid. Record any rejected plaintext/cross-origin flows without restoring
      unsafe redirects.
- [ ] Capabilities, assets, bridge and release notes succeed with correctly
      matching TLS certificates. Record certificate-name incompatibilities.
- [ ] Walk, fly, teleport, cross regions, render avatars, attachments, terrain,
      mesh/materials, lighting, water, particles and media.
- [ ] Chat, IM, groups, friends, voice/device switching and notifications behave
      as expected. Verify Windows SLURL handoff targets Amante, not Aurora.
- [ ] Verify inventory create/edit/delete, uploads, scripting, build permissions,
      land/estate controls and Marketplace flows with explicit owner consent.
      Avoid destructive actions or purchases merely to mark a test complete.
- [ ] Check Blender integration using an authorized local installation.
- [ ] Repeat important paths with an existing **Amante** profile; no automatic
      Aurora credential/profile migration is expected.

Keep screenshots/log extracts locally and redact them before sharing. Report
pass/fail/not-applicable per item; leave unexercised features untested.
