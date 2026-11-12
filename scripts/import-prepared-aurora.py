"""Import an explicitly prepared upstream merge and regenerate its byte inventory.

The prepared tree must be a Git checkout containing the official release tag.
Renderer, assets, network, world and wearables must match that tag exactly.
This is source synchronization only: it does not publish a build or a download.
"""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tarfile

prepared = Path(sys.argv[1]).resolve()
tag = sys.argv[2]
destination = Path(sys.argv[3] if len(sys.argv) > 3 else "amante-viewer").resolve()


def git(*args):
    return subprocess.check_output(["git", "-C", str(prepared), *args])


def digest(data):
    return hashlib.sha256(data).hexdigest()


assert not git("ls-files", "--unmerged"), "Resolve and stage the merge first"
commit = git("rev-parse", f"{tag}^{{commit}}").decode().strip()
releases = {
    "v0.9.1": "76f7cd150bb6238fbe5eb895e47ee84777896dfe",
    "v0.9.2": "fde838f1234ebfa2848445965f1f90541f0652f6",
    "v0.9.3": "7851d13e61b3f83c7a8ff367f7678fbfd2e3c156",
    "v0.9.6": "019d41a5ba4b87fb2f6e7072ae5c003b31017c26",
    "v0.9.8": "e71e818f136a79219b675a55ea0e4a0ad9996409",
}
assert releases.get(tag) == commit, "Unexpected release"
metadata = {"amante-upstream-manifest.json", "amante-security-manifest.json"}
files = set(git("ls-files", "-z").decode().strip("\0").split("\0")) - metadata
assert all(not Path(p).is_absolute() and ".." not in Path(p).parts for p in files)

# Keep localization keys and protocol identifiers intact. The release's new
# product-owned copy follows the existing Amante identity; author credits do not.
for locale in ("en", "fr", "ja"):
    path = prepared / f"crates/aurora-i18n/locales/{locale}.ftl"
    lines = path.read_text().splitlines(keepends=True)
    key = ""
    for i, line in enumerate(lines):
        if line and not line[0].isspace() and "=" in line:
            key = line.split("=", 1)[0]
        if not line.lstrip().startswith("#") and not any(word in key for word in ("credit", "author", "license")):
            lines[i] = line.replace("Aurora", "Amante")
    path.write_text("".join(lines))

protected = (
    "crates/aurora-render/", "crates/aurora-net/", "crates/aurora-assets/",
    "crates/aurora-viewer/src/scene/", "crates/aurora-viewer/src/world/",
    "crates/aurora-viewer/src/wearable_editor/",
)
security_files = {
    "scripts/package.ps1", "Cargo.lock", "crates/aurora-viewer/Cargo.toml",
    "crates/aurora-viewer/src/ui/emoji.rs",
    "crates/aurora-viewer/src/ui/fonts.rs",
    "crates/aurora-viewer/src/ui/fonts/fallback.rs",
    "crates/aurora-viewer/src/ui/fonts/preview.rs",
    "crates/aurora-viewer/src/system_notify.rs",
    "crates/aurora-viewer/src/slurl_handoff.rs",
}
security_prefixes = ("crates/amante-fonts/", "crates/amante-credentials/")
manifest = {"upstream": commit, "release": tag, "files": {}, "added": {}}
security = {"upstream": commit, "files": {}, "added": {}}
upstream_names = set()
archive = subprocess.Popen(["git", "-C", str(prepared), "archive", tag], stdout=subprocess.PIPE)
with tarfile.open(fileobj=archive.stdout, mode="r|") as tree:
    for entry in tree:
        if not entry.isfile():
            continue
        name = entry.name
        upstream_names.add(name)
        assert name in files, f"Unreviewed upstream deletion: {name}"
        original = tree.extractfile(entry).read()
        actual = (prepared / name).read_bytes()
        if name.startswith(protected) or name == "Cargo.toml":
            assert actual == original, f"Protected runtime differs from official {tag}: {name}"
        original_hash, actual_hash = digest(original), digest(actual)
        baseline = original_hash if name in security_files else actual_hash
        manifest["files"][name] = {"upstream": original_hash, "amante": baseline}
        if name in security_files:
            security["files"][name] = {"base": baseline, "sha256": actual_hash}
assert archive.wait() == 0
for name in sorted(files - upstream_names):
    assert not name.startswith(protected), f"Extra protected runtime file: {name}"
    inventory = security if name.startswith(security_prefixes) else manifest
    inventory["added"][name] = digest((prepared / name).read_bytes())

# Preserve build caches and untracked user data. Remove only previously tracked
# vendor source files that the prepared merge deliberately removed.
old = subprocess.check_output(["git", "-C", str(destination), "ls-files", "-z", "."]).decode().strip("\0").split("\0")
for relative in filter(None, old):
    if relative not in files | metadata:
        (destination / relative).unlink(missing_ok=True)
for name in sorted(files):
    source, target = prepared / name, destination / name
    data = source.read_bytes()
    if not target.exists() or target.read_bytes() != data:
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)
    target.chmod(source.stat().st_mode & 0o777)
for name, value in (("amante-upstream-manifest.json", manifest), ("amante-security-manifest.json", security)):
    (destination / name).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")
print(f"Imported {tag} ({commit}); {len(upstream_names)} official files; protected runtime byte-identical.")
