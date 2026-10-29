import { test } from "node:test";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync, spawnSync } from "node:child_process";
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, copyFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const verifier = fileURLToPath(new URL("./verify-viewer-preservation.mjs", import.meta.url));
const blob = (bytes) => createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex");

function fixture(run) {
  const root = mkdtempSync(join(tmpdir(), "amante-preservation-"));
  const native = join(root, "amante-viewer");
  const text = Buffer.from("synthetic source\nsecond line\n");
  const binary = Buffer.from([0, 13, 10, 255, 1]);
  try {
    mkdirSync(join(root, "scripts"));
    mkdirSync(join(native, "docs/amante"), { recursive: true });
    mkdirSync(join(native, "assets"));
    copyFileSync(verifier, join(root, "scripts/verify.mjs"));
    writeFileSync(join(native, ".gitattributes"), "*.bin binary\n");
    writeFileSync(join(native, "module.rs"), text.toString().replaceAll("\n", "\r\n"));
    writeFileSync(join(native, "assets/fixture.bin"), binary);
    writeFileSync(join(native, "docs/amante/upstream-inventory.json"), JSON.stringify({
      baseline: "synthetic-baseline",
      files: [{ path: "module.rs", blob: blob(text) }, { path: "assets/fixture.bin", blob: blob(binary) }],
      uiModules: [{ path: "module.rs" }],
      menuRegistrations: [], settingFields: [], keybindingAndToolbarDeclarations: [],
    }));
    const git = (...args) => execFileSync("git", ["-C", root, ...args], { stdio: "pipe" });
    git("init", "--quiet");
    git("add", "amante-viewer");
    git("-c", "user.name=Synthetic Test", "-c", "user.email=synthetic@example.invalid", "commit", "--quiet", "-m", "Synthetic fixture");
    const execute = (...args) => spawnSync(process.execPath, [join(root, "scripts/verify.mjs"), ...args], { encoding: "utf8" });
    const report = () => JSON.parse(readFileSync(join(root, "generated-artifacts/source-preservation.json"), "utf8"));
    run({ root, native, git, execute, report });
  } finally { rmSync(root, { recursive: true, force: true }); }
}

test("Windows checkout line endings do not masquerade as source or binary edits", () => fixture(({ execute, report }) => {
  const result = execute("--require-tracked");
  assert.equal(result.status, 0, result.stderr);
  assert.equal(report().unchanged, 2);
  assert.equal(report().checkoutLineEndingDifferences, 1);
  assert.deepEqual(report().changedAssets, []);
  assert.deepEqual(report().untrackedBaseline, []);
}));

test("real binary asset changes remain visible", () => fixture(({ native, execute, report }) => {
  writeFileSync(join(native, "assets/fixture.bin"), Buffer.from([0, 10, 255, 1]));
  assert.equal(execute("--require-tracked").status, 0);
  assert.equal(report().modified, 1);
  assert.deepEqual(report().changedAssets, ["assets/fixture.bin"]);
}));

test("present but untracked baseline source is rejected", () => fixture(({ git, execute, report }) => {
  git("rm", "--cached", "amante-viewer/module.rs");
  assert.equal(execute("--require-tracked").status, 1);
  assert.deepEqual(report().untrackedBaseline, ["module.rs"]);
  assert.deepEqual(report().missing, []);
}));

test("missing baseline files are reported and fail the check", () => fixture(({ native, execute, report }) => {
  rmSync(join(native, "module.rs"));
  rmSync(join(native, "assets/fixture.bin"));
  assert.equal(execute().status, 1);
  assert.deepEqual(report().missing, ["module.rs", "assets/fixture.bin"]);
}));

test("build/package evidence never becomes a desktop test result", () => fixture(({ root, native, git, execute }) => {
  const path = join(root, "synthetic-evidence.json");
  writeFileSync(path, JSON.stringify({
    schema: 1, nativeBuild: "passed", packageVerification: "passed",
    revision: git("rev-parse", "HEAD").toString().trim(), sha256: "a".repeat(64),
  }));
  assert.equal(execute("--require-tracked", path).status, 0);
  const csv = readFileSync(join(native, "docs/amante/feature-parity.csv"), "utf8");
  assert.match(csv, /complete native target built; not a per-item test/);
  assert.match(csv, /distribution verified; not a per-item test/);
  assert.match(csv, /not runtime-tested/);
  writeFileSync(path, "{}");
  assert.notEqual(execute(path).status, 0);
}));

test("an unrelated package revision cannot attest this source", () => fixture(({ root, execute }) => {
  const path = join(root, "synthetic-evidence.json");
  writeFileSync(path, JSON.stringify({
    schema: 1, nativeBuild: "passed", packageVerification: "passed",
    revision: "0".repeat(40), sha256: "a".repeat(64),
  }));
  const result = execute(path);
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /different source revision/);
}));

test("native edits after packaging cannot inherit its evidence", () => fixture(({ root, native, git, execute }) => {
  const path = join(root, "synthetic-evidence.json");
  writeFileSync(path, JSON.stringify({
    schema: 1, nativeBuild: "passed", packageVerification: "passed",
    revision: git("rev-parse", "HEAD").toString().trim(), sha256: "a".repeat(64),
  }));
  writeFileSync(join(native, "module.rs"), "actual changed source\n");
  const result = execute(path);
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /Native source differs/);
}));
