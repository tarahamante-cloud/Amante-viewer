"""Create a clean, reproducible branding-only Amante copy of an Aurora snapshot.

Usage: python3 scripts/rebuild-amante-from-upstream.py SNAPSHOT DESTINATION BRAND_SOURCE
DESTINATION must be a directory named amante-viewer. No user profile is touched.
"""
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys

UPSTREAM = "b21e5c27880176978d4a301220645c2830fef3a9"
OWNER = "tarahamante-cloud/Amante-viewer"
PROTECTED = [
    "crates/aurora-render/", "crates/aurora-net/", "crates/aurora-assets/",
    "crates/aurora-viewer/src/scene/", "crates/aurora-viewer/src/world/",
    "crates/aurora-viewer/src/wearable_editor/",
]
snapshot, destination, branding = map(lambda value: Path(value).resolve(), sys.argv[1:])
assert destination.name == "amante-viewer" and snapshot.is_dir()
assert (snapshot / "crates/aurora-viewer/src/scene/textures.rs").is_file()
assert not (destination / ".git").exists(), "Never remove an independent Git checkout"


def digest(data):
    return hashlib.sha256(data).hexdigest()


def block(text, marker):
    start = text.index(marker)
    opening = text.index("{", start)
    depth = 1
    end = opening + 1
    while depth:
        depth += (text[end] == "{") - (text[end] == "}")
        end += 1
    return text[start:end]


def brand_asset(previous, clean):
    path = branding / previous
    return (path if path.is_file() else branding / clean).read_bytes()


brand_files = {
    "assets/branding/aurora-logo.png": brand_asset("crates/aurora-viewer/assets/amante-crest.png", "assets/branding/aurora-logo.png"),
    "assets/branding/amante-crest-icon.png": brand_asset("crates/aurora-viewer/assets/amante-crest-icon.png", "assets/branding/amante-crest-icon.png"),
    "assets/branding/aurora.ico": brand_asset("assets/branding/amante.ico", "assets/branding/aurora.ico"),
}
old_theme = (branding / "crates/aurora-viewer/src/theme.rs").read_text()
old_loading = (branding / "crates/aurora-viewer/src/ui/loading.rs").read_text()
crest_start = old_loading.index("    // Large, centered crest")
crest_end = old_loading.index("    // brand, top left", crest_start)
crest_overlay = old_loading[crest_start:crest_end]
baseline = {
    str(path.relative_to(snapshot)): digest(path.read_bytes())
    for path in snapshot.rglob("*") if path.is_file()
}

# Old binaries/cache are not source and must not enter Git or the new package.
# Keep a local Cargo build cache, if present, without copying it into the snapshot.
destination.mkdir(parents=True, exist_ok=True)
for child in destination.iterdir():
    if child.name == "target":
        continue
    if child.is_dir() and not child.is_symlink():
        shutil.rmtree(child)
    else:
        child.unlink()
shutil.copytree(snapshot, destination, dirs_exist_ok=True)

# Replace product strings, not crate names, asset formats, protocol identifiers,
# network messages, demo residents or copyright/attribution notices.
for directory in ["crates/aurora-viewer", "crates/aurora-version", "crates/aurora-updater"]:
    for path in (destination / directory).rglob("*.rs"):
        if any(str(path.relative_to(destination)).startswith(prefix) for prefix in PROTECTED):
            continue
        text = path.read_text()
        changed = text.replace("Aurora Viewer", "Amante Viewer").replace("AuroraViewer/", "AmanteViewer/")
        if path.name != "build_support.rs":
            changed = changed.replace('"aurora-viewer.exe"', '"amante-viewer.exe"')
        if directory == "crates/aurora-updater":
            changed = changed.replace("Aurora-Viewer-", "Amante-Viewer-")
        if changed != text:
            path.write_text(changed)

for path in (destination / "crates/aurora-i18n/locales").glob("*.ftl"):
    text = path.read_text()
    # Preserve Fluent keys and placeholders exactly, translate the product
    # identity in values. Do not relabel upstream people or demo regions.
    text = re.sub(
        r"(?m)^([a-z0-9-]+\s*=\s*|[ \t]+)(.*)$",
        lambda match: match[1] + (
            match[2] if any(word in match[2] for word in ["Aurora Builders", "Aurora Demo", "Aurora Démo"])
            else match[2].replace("Aurora", "Amante")
        ),
        text,
    )
    path.write_text(text)

def edit(relative, transform):
    path = destination / relative
    text = path.read_text()
    changed = transform(text)
    assert changed != text, f"No change for expected branding edit: {relative}"
    path.write_text(changed)

edit("crates/aurora-viewer/src/settings.rs", lambda s: s.replace(
    'ProjectDirs::from("org", "Aurora", "AuroraViewer")',
    'ProjectDirs::from("org", "Amante", "AmanteViewerAlpha")',
))
edit("crates/aurora-viewer/src/theme.rs", lambda s: s.replace(
    block(s, "impl Default for Theme"), block(old_theme, "impl Default for Theme")
).replace(
    block(s, "pub fn aurora_gradient("), block(old_theme, "pub fn aurora_gradient(")
))
edit("crates/aurora-viewer/src/ui/loading.rs", lambda s: s.replace(
    "    // brand, top left", crest_overlay + "    // brand, top left", 1
))
edit("crates/aurora-viewer/src/ui/icons.rs", lambda s: s.replace(
    'const BRAND_SVGS: &[(&str, &[u8])] = &[("wolf", include_bytes!("../../../../assets/branding/aurora-logo-white.svg"))];',
    'const BRAND_SVGS: &[(&str, &[u8])] = &[];',
).replace(
    'const PNGS: &[(&str, &[u8])] = &[',
    'const PNGS: &[(&str, &[u8])] = &[("wolf", include_bytes!("../../../../assets/branding/amante-crest-icon.png")), ',
))

# Ownership configuration must not install Aurora over Amante, file Amante
# reports to Aurora, or use Aurora's OAuth app. All algorithms remain upstream.
for relative in [
    "crates/aurora-updater/src/feed.rs",
    "crates/aurora-viewer/src/links.rs",
    "crates/aurora-viewer/src/bug_report/github.rs",
    "crates/aurora-viewer/src/crash/github.rs",
]:
    path = destination / relative
    text = path.read_text().replace("Aurora-Viewer/Aurora-Viewer", OWNER)
    text = text.replace("Iv23liT7PLWClMLztI1W", "")
    if relative.endswith("links.rs"):
        text = text.replace(
            "https://raw.githubusercontent.com/" + OWNER + "/main/news/",
            "https://raw.githubusercontent.com/" + OWNER + "/main/amante-viewer/news/",
        )
        text = re.sub(r'pub const DISCORD: &str = "[^"]*";', 'pub const DISCORD: &str = "";', text)
        text = text.replace("https://aurora-viewer.org/privacy/", "https://github.com/" + OWNER + "/blob/main/amante-viewer/AMANTE.md")
        text = text.replace("https://aurora-viewer.org", "https://github.com/" + OWNER + "/releases")
    path.write_text(text)

for relative, data in brand_files.items():
    path = destination / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)

# Upstream packaging with only distribution-name changes; no old native code,
# dependency patches, texture workarounds, or third-party plugins are inherited.
edit("scripts/package.ps1", lambda s: s.replace(
    "Copy-Item -LiteralPath $Exe -Destination $Out -ErrorAction Stop",
    "$binaryName = if ([IO.Path]::GetExtension($Exe) -eq '.exe') { 'amante-viewer.exe' } else { 'amante-viewer' }\n"
    "Copy-Item -LiteralPath $Exe -Destination (Join-Path $Out $binaryName) -ErrorAction Stop",
).replace("^Aurora Viewer ", "^Amante Viewer ").replace(
    "$docs = 'LICENSE', 'NOTICE.md', 'README.md'",
    "$docs = 'LICENSE', 'NOTICE.md', 'README.md', 'AMANTE.md', 'amante-upstream-manifest.json'",
))

(destination / "AMANTE.md").write_text(
    "# Amante Viewer Alpha — clean upstream rebuild\n\n"
    f"Based on Aurora Viewer commit `{UPSTREAM}`.\n\n"
    "The runtime is upstream Aurora with Amante name, crest, palette, product "
    "directories, owner links and package identity. The previous Amante-specific "
    "renderer, texture, clothing, browser, toolbar and dependency changes are not included.\n\n"
    "LICENSE and NOTICE.md retain Aurora and third-party attribution. "
    "The source manifest records every original and branded file hash.\n\n"
    "This clean Alpha uses its own fresh profile and cache, so it does not import "
    "settings or cached textures from earlier Amante builds. Add your account again "
    "on first launch. No old credentials, profiles or user caches are deleted. Reports and updates use the Amante "
    "repository. Direct OAuth reporting is unconfigured rather than using Aurora's app. "
    "Existing user skins remain user choices. Native build/test checks do not prove "
    "texture loading in a live Second Life session.\n"
)
# Format only branding edits. In particular never reformat the renderer,
# networking, texture streamer or world code being preserved byte-for-byte.
rust_files = [
    str(destination / name) for name, original_hash in baseline.items()
    if name.endswith(".rs") and digest((destination / name).read_bytes()) != original_hash
]
subprocess.run([
    os.environ.get("AMANTE_RUSTFMT", "rustfmt"), "--edition", "2024",
    "--config-path", str(destination / "rustfmt.toml"), "--config", "skip_children=true",
    *rust_files,
], check=True)
manifest = {"upstream": UPSTREAM, "files": {}, "added": {}}
for relative, upstream_hash in baseline.items():
    file = destination / relative
    assert file.is_file(), f"Missing upstream file: {relative}"
    final_hash = digest(file.read_bytes())
    manifest["files"][relative] = {"upstream": upstream_hash, "amante": final_hash}
for directory, subdirs, filenames in os.walk(destination):
    if Path(directory) == destination:
        subdirs[:] = [name for name in subdirs if name != "target"]
    for name in filenames:
        path = Path(directory) / name
        relative = str(path.relative_to(destination))
        if relative not in baseline:
            manifest["added"][relative] = digest(path.read_bytes())
for relative, hashes in manifest["files"].items():
    if any(relative.startswith(prefix) for prefix in PROTECTED):
        assert hashes["upstream"] == hashes["amante"], f"Runtime changed: {relative}"
assert manifest["files"]["Cargo.lock"]["upstream"] == manifest["files"]["Cargo.lock"]["amante"]
assert manifest["files"]["Cargo.toml"]["upstream"] == manifest["files"]["Cargo.toml"]["amante"]
(destination / "amante-upstream-manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
changed = [name for name, hashes in manifest["files"].items() if hashes["upstream"] != hashes["amante"]]
print(json.dumps({"upstream": UPSTREAM, "original_files": len(baseline), "branding_files": changed, "added": list(manifest["added"])}, indent=2))
