// Compare the native tree against the immutable, pre-edit source inventory.
// This verifies source preservation, not successful behavior on a live grid.
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const workspace = fileURLToPath(new URL("../", import.meta.url));
const root = join(workspace, "amante-viewer");
const inventory = JSON.parse(readFileSync(join(root, "docs/amante/upstream-inventory.json"), "utf8"));
// Compare canonical Git content, not Windows checkout CRLF bytes. Apply Git's
// text/binary attributes in one process; never normalize binary assets by hand.
const present = inventory.files.filter((file) => existsSync(join(root, file.path)));
const hashes = present.length ? execFileSync("git", ["-c", "core.autocrlf=input", "hash-object", "--stdin-paths"], {
  cwd: workspace,
  input: present.map((file) => JSON.stringify(`amante-viewer/${file.path}`)).join("\n") + "\n",
  encoding: "utf8",
  maxBuffer: 4 * 1024 * 1024,
}).trim().split(/\r?\n/) : [];
if (hashes.length !== present.length || hashes.some((hash) => !/^[a-f0-9]{40}$/.test(hash))) {
  throw new Error("Git did not produce one canonical SHA-1 blob identity per baseline file");
}
const canonical = new Map(present.map((file, i) => [file.path, hashes[i]]));
const records = inventory.files.map((file) => {
  const path = join(root, file.path);
  if (!existsSync(path)) return { path: file.path, status: "missing" };
  const bytes = readFileSync(path);
  // Git's upstream object identifier format, not a signature/security primitive.
  // Production release authenticity must not use this comparison as trust.
  const blob = createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex");
  const currentBlob = blob === file.blob ? blob : canonical.get(file.path);
  return {
    path: file.path, status: currentBlob === file.blob ? "unchanged" : "modified",
    baselineBlob: file.blob, currentBlob, workingTreeBlob: blob,
    checkoutLineEndingsOnly: blob !== file.blob && currentBlob === file.blob,
  };
});
const byPath = new Map(records.map((r) => [r.path, r.status]));
const args = process.argv.slice(2);
const requireTracked = args.includes("--require-tracked");
const evidencePath = args.find((arg) => arg !== "--require-tracked");
const tracked = requireTracked
  ? new Set(execFileSync("git", ["-C", workspace, "ls-files", "-z", "--", "amante-viewer"], { encoding: "utf8" }).split("\0"))
  : null;
const untrackedBaseline = tracked ? inventory.files.filter((file) => !tracked.has(`amante-viewer/${file.path}`)).map((file) => file.path) : [];
// Optional evidence is emitted only after checking the extracted Windows ZIP.
// Whole-target compilation and distribution checks are NOT per-feature tests.
const evidence = evidencePath ? JSON.parse(readFileSync(evidencePath, "utf8")) : null;
if (evidence && (evidence.schema !== 1 || evidence.nativeBuild !== "passed" || evidence.packageVerification !== "passed" || !/^[a-f0-9]{64}$/.test(evidence.sha256))) {
  throw new Error("Expected successful Windows package verification evidence");
}
if (evidence) {
  const git = (...args) => execFileSync("git", ["-C", workspace, ...args], { encoding: "utf8" }).trim();
  if (evidence.revision !== git("rev-parse", "HEAD")) {
    throw new Error("Package evidence belongs to a different source revision");
  }
  // The generated CSV changes during this very check. All other tracked and
  // non-ignored native files must still be the exact built source snapshot.
  const generatedCsv = "amante-viewer/docs/amante/feature-parity.csv";
  const dirty = [
    ...git("diff", "--name-only", "HEAD", "--", "amante-viewer").split("\n"),
    ...git("ls-files", "--others", "--exclude-standard", "--", "amante-viewer").split("\n"),
  ].filter((path) => path && path.replaceAll("\\", "/") !== generatedCsv);
  if (dirty.length) throw new Error(`Native source differs from packaged revision: ${dirty.join(", ")}`);
}
const rows = [["kind", "baseline source", "declaration", "source status", "Windows build status", "Windows package status", "desktop status", "build revision", "package SHA-256"]];
const row = (kind, path, declaration) => rows.push([
  kind, path, declaration, byPath.get(path.replace(/:\d+$/, "")),
  evidence ? "complete native target built; not a per-item test" : "not Windows-verified",
  evidence ? "distribution verified; not a per-item test" : "not Windows-verified",
  "not runtime-tested", evidence?.revision ?? "", evidence?.sha256 ?? "",
]);
for (const file of inventory.uiModules) row("UI module", file.path, "");
for (const entry of inventory.menuRegistrations) row("menu registration call site", entry.id, entry.context);
for (const entry of inventory.settingFields) row("setting declaration", entry.id, `${entry.name}: ${entry.type}`);
for (const entry of inventory.keybindingAndToolbarDeclarations) row("keybinding/toolbar declaration", entry.id, entry.source);
const csv = rows.map((fields) => fields.map((value) => `"${String(value ?? "").replaceAll('"', '""')}"`).join(",")).join("\n");
writeFileSync(join(root, "docs/amante/feature-parity.csv"), `${csv}\n`);
const summary = {
  baseline: inventory.baseline,
  originalFiles: records.length,
  unchanged: records.filter((r) => r.status === "unchanged").length,
  modified: records.filter((r) => r.status === "modified").length,
  checkoutLineEndingDifferences: records.filter((r) => r.checkoutLineEndingsOnly).length,
  missing: records.filter((r) => r.status === "missing").map((r) => r.path),
  ...(requireTracked ? { untrackedBaseline } : {}),
  changedAssets: records.filter((r) => r.status !== "unchanged" && /(?:^|\/)assets\//.test(r.path)).map((r) => r.path),
  evidence: "Canonical Git blob comparison against the pre-edit inventory; checkout line endings recorded separately; not runtime parity",
};
mkdirSync(join(workspace, "generated-artifacts"), { recursive: true });
writeFileSync(join(workspace, "generated-artifacts/source-preservation.json"), `${JSON.stringify({ ...summary, files: records }, null, 2)}\n`);
console.log(JSON.stringify(summary, null, 2));
if (summary.missing.length || untrackedBaseline.length) process.exitCode = 1;
