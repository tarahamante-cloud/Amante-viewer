#!/usr/bin/env python3
"""Run actual palette/accessibility suites without linking the whole viewer.

Requires dependencies already compiled by the native CI-profile build/check.
The full Windows harness remains authoritative for the complete application.
"""
import os
from pathlib import Path
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "amante-viewer/crates/aurora-viewer/src"
DEPS = ROOT / "amante-viewer/target/ci/deps"


def library(name):
    candidates = list(DEPS.glob(f"lib{name}-*.rlib"))
    if not candidates:
        raise SystemExit(f"Missing {name}: compile the native CI-profile dependencies first.")
    return max(candidates, key=lambda path: path.stat().st_mtime)


def main():
    theme = (SOURCE / "theme.rs").read_text()
    # Use the production types/defaults and palette resolution verbatim, leaving
    # out only egui installation code that depends on the rest of the viewer.
    types = theme[:theme.index("impl Theme {")]
    methods = theme[theme.index("    pub fn palette("):theme.index("    pub fn apply(")]
    modules = []
    for name in ("high_contrast", "interface_colors", "colorblind"):
        production = (SOURCE / (name + ".rs")).read_text()
        if name == "high_contrast":
            # Its separate settings-default tests require the entire settings
            # graph. This focused runner executes only the two color suites.
            production = production.replace("#[cfg(test)]\nmod tests;", "")
        else:
            production = production.replace(
                "mod tests;", f'#[path = "{SOURCE / name / "tests.rs"}"] mod tests;'
            )
        modules.append(f"mod {name} {{\n{production}\n}}")
    harness = (
        "mod settings { pub type Rgba = [u8; 4]; }\n"
        f"mod theme {{\n{types}\nimpl Theme {{\n{methods}\n}}\n}}\n"
        + "\n".join(modules) + "\n"
    )
    env = os.environ.copy()
    env.pop("LD_AUDIT", None)
    with tempfile.TemporaryDirectory(prefix="amante-palette-") as temporary:
        source = Path(temporary) / "palette.rs"
        binary = Path(temporary) / "palette-tests"
        source.write_text(harness)
        command = [
            "rustup", "run", "1.99.1", "rustc", "--edition=2024", "--test",
            str(source), "-o", str(binary), "-L", f"dependency={DEPS}",
            "-A", "dead_code", "-A", "unused_imports",
        ]
        for name in ("egui", "serde", "serde_json", "aurora_i18n"):
            command += ["--extern", f"{name}={library(name)}"]
        subprocess.run(command, env=env, check=True)
        subprocess.run([str(binary), "--test-threads=1"], env=env, check=True)


if __name__ == "__main__":
    main()
