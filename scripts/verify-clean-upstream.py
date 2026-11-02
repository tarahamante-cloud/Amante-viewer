"""Check the complete native source against its recorded clean upstream import."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

root = Path(sys.argv[1] if len(sys.argv) > 1 else "amante-viewer")
manifest = json.loads((root / "amante-upstream-manifest.json").read_text())
protected = [
    "crates/aurora-render/", "crates/aurora-net/", "crates/aurora-assets/",
    "crates/aurora-viewer/src/scene/", "crates/aurora-viewer/src/world/",
    "crates/aurora-viewer/src/wearable_editor/",
]
original = 0
for name, hashes in manifest["files"].items():
    data = (root / name).read_bytes()
    assert hashlib.sha256(data).hexdigest() == hashes["amante"], f"Changed import: {name}"
    if name in ("Cargo.lock", "Cargo.toml") or any(name.startswith(p) for p in protected):
        assert hashes["amante"] == hashes["upstream"], f"Runtime differs from Aurora: {name}"
        original += 1
for name, expected in manifest["added"].items():
    assert hashlib.sha256((root / name).read_bytes()).hexdigest() == expected, f"Changed branding asset: {name}"
if "--require-tracked" in sys.argv:
    tracked = set(subprocess.check_output(["git", "-C", str(root), "ls-files", "-z", "."]).decode().split("\0"))
    expected = set(manifest["files"]) | set(manifest["added"]) | {"amante-upstream-manifest.json"}
    assert not expected - tracked, f"Source files missing from Git: {sorted(expected - tracked)[:10]}"
print(f"Verified {len(manifest['files'])} imported files; {original} protected runtime/dependency files are identical to Aurora {manifest['upstream']}.")
