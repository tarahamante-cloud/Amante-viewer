// Compare the native tree against the immutable, pre-edit source inventory.
// This verifies source preservation, not successful behavior on a live grid.
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const workspace = fileURLToPath(new URL("../", import.meta.url));
const root = join(workspace, "amante-viewer");
const inventory = JSON.parse(readFileSync(join(root, "docs/amante/upstream-inventory.json"), "utf8"));
const records = inventory.files.map((file) => {
  const path = join(root, file.path);
  if (!existsSync(path)) return { path: file.path, status: "missing" };
  const bytes = readFileSync(path);
  // Git's upstream object identifier format, not a signature/security primitive.
  // Production release authenticity must not use this comparison as trust.
  const blob = createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex");
  return { path: file.path, status: blob === file.blob ? "unchanged" : "modified", baselineBlob: file.blob, currentBlob: blob };
});
const byPath = new Map(records.map((r) => [r.path, r.status]));
// Optional evidence is emitted only after checking the extracted Windows ZIP.
// Whole-target compilation and distribution checks are NOT per-feature tests.
const evidence = process.argv[2] ? JSON.parse(readFileSync(process.argv[2], "utf8")) : null;
if (evidence && (evidence.schema !== 1 || evidence.nativeBuild !== "passed" || evidence.packageVerification !== "passed" || !/^[a-f0-9]{64}$/.test(evidence.sha256))) {
  throw new Error("Expected successful Windows package verification evidence");
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
  missing: records.filter((r) => r.status === "missing").map((r) => r.path),
  changedAssets: records.filter((r) => r.status !== "unchanged" && /(?:^|\/)assets\//.test(r.path)).map((r) => r.path),
  evidence: "Git blob comparison against the pre-edit inventory; not runtime parity",
};
mkdirSync(join(workspace, "generated-artifacts"), { recursive: true });
writeFileSync(join(workspace, "generated-artifacts/source-preservation.json"), `${JSON.stringify({ ...summary, files: records }, null, 2)}\n`);
console.log(JSON.stringify(summary, null, 2));
if (summary.missing.length) process.exitCode = 1;
