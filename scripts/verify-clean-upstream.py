"""Check the complete native source against its recorded clean upstream import."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tomllib

root = Path(sys.argv[1] if len(sys.argv) > 1 else "amante-viewer")
manifest = json.loads((root / "amante-upstream-manifest.json").read_text())
protected = [
    "crates/aurora-render/", "crates/aurora-net/", "crates/aurora-assets/",
    "crates/aurora-viewer/src/scene/", "crates/aurora-viewer/src/world/",
    "crates/aurora-viewer/src/wearable_editor/",
]
original = 0
security_updated = 0
security_path = root / "amante-security-manifest.json"
security = json.loads(security_path.read_text()) if security_path.exists() else {"files": {}, "added": {}}
# The user separately authorized these dependency migrations after the clean
# reset. Never allow this inventory to become an arbitrary runtime override.
allowed_security_files = {
    "scripts/package.ps1",
    "Cargo.lock", "crates/aurora-viewer/Cargo.toml",
    "crates/aurora-viewer/src/ui/emoji.rs",
    "crates/aurora-viewer/src/ui/fonts.rs",
    "crates/aurora-viewer/src/ui/fonts/fallback.rs",
    "crates/aurora-viewer/src/ui/fonts/preview.rs",
    "crates/aurora-viewer/src/system_notify.rs",
    "crates/aurora-viewer/src/slurl_handoff.rs",
}
allowed_security_added = {
    "crates/amante-fonts/Cargo.toml", "crates/amante-fonts/src/lib.rs",
    "crates/amante-credentials/Cargo.toml", "crates/amante-credentials/src/lib.rs",
    "crates/amante-credentials/tests/native_store.rs",
    "crates/amante-credentials/tests/linux_bus.rs",
}
assert set(security["files"]) <= allowed_security_files, "Unapproved security override"
assert set(security["added"]) <= allowed_security_added, "Unapproved security addition"
if security_path.exists():
    assert security["upstream"] == manifest["upstream"], "Security inventory baseline mismatch"


def matches(data, expected):
    if hashlib.sha256(data).hexdigest() == expected:
        return True
    # Git's Windows checkout converts text to CRLF. Permit that conversion
    # only, never arbitrary whitespace changes or binary normalization.
    if b"\r\n" not in data or b"\0" in data:
        return False
    try:
        data.decode("utf-8")
    except UnicodeDecodeError:
        return False
    return hashlib.sha256(data.replace(b"\r\n", b"\n")).hexdigest() == expected


for name, hashes in manifest["files"].items():
    data = (root / name).read_bytes()
    if name in security["files"]:
        patch = security["files"][name]
        assert patch["base"] == hashes["amante"], f"Changed security baseline: {name}"
        assert matches(data, patch["sha256"]), f"Changed security migration: {name}"
        if name == "Cargo.lock":
            packages = tomllib.loads(data.decode())["package"]
            versions = {p["name"]: p["version"] for p in packages}
            assert not set(versions) & {"ttf-parser", "owned_ttf_parser", "ab_glyph", "zvariant", "zbus_names", "zvariant_derive", "secret-service"}, "Unmaintained dependency reintroduced"
            for package, version in {"ctutils": "0.4.7", "rustls-webpki": "0.103.16", "pulldown-cmark": "0.13.6"}.items():
                found = [p["version"] for p in packages if p["name"] == package]
                # ctutils' only remaining consumer disappeared with the old
                # saved-login backend. Absence resolves its advisory too.
                assert (bool(found) or package == "ctutils") and all(v == version for v in found), f"Missing approved security patch: {package}"
        security_updated += 1
        continue
    assert matches(data, hashes["amante"]), f"Changed import: {name}"
    if name in ("Cargo.lock", "Cargo.toml") or any(name.startswith(p) for p in protected):
        assert hashes["amante"] == hashes["upstream"], f"Runtime differs from Aurora: {name}"
        original += 1
for name, expected in manifest["added"].items():
    assert matches((root / name).read_bytes(), expected), f"Changed branding asset: {name}"
for name, expected in security["added"].items():
    assert matches((root / name).read_bytes(), expected), f"Changed security addition: {name}"
if "--require-tracked" in sys.argv:
    tracked = set(subprocess.check_output(["git", "-C", str(root), "ls-files", "-z", "."]).decode().split("\0"))
    expected = set(manifest["files"]) | set(manifest["added"]) | {"amante-upstream-manifest.json"}
    expected |= set(security["added"])
    if security_path.exists():
        expected.add("amante-security-manifest.json")
    assert not expected - tracked, f"Source files missing from Git: {sorted(expected - tracked)[:10]}"
print(f"Verified {len(manifest['files'])} imported files; {original} protected runtime/dependency files are identical to Aurora {manifest['upstream']}; {security_updated} explicitly inventoried security migrations.")
