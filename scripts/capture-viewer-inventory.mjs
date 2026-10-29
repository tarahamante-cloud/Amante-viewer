// Source-level parity evidence, not a substitute for desktop behavior tests.
// Read the immutable upstream Git objects, never the edited working tree.
import { execFileSync } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve("amante-viewer");
const baseline = "d555fb677004bee8fd9e7f12b0e33a8978e77d4c";
// Replit checkpoints import a nested checkout into the project repository;
// they do not retain its nested .git directory. Keep an immutable object store
// outside the workspace, or pass the original upstream clone as argv[2].
const objectStore = process.argv[2] || "/tmp/aurora-baseline.git";
const git = (...args) => execFileSync("git", [`--git-dir=${objectStore}`, ...args], {
  encoding: "utf8",
  maxBuffer: 64 * 1024 * 1024,
});
const files = git("ls-tree", "-rl", baseline).trim().split("\n").map((row) => {
  const match = row.match(/^(\d+) blob ([a-f0-9]+)\s+(\d+)\t(.+)$/);
  if (!match) throw new Error(`Unexpected Git tree entry: ${row}`);
  return { path: match[4], mode: match[1], blob: match[2], bytes: Number(match[3]) };
});
const source = files.filter((f) => /^crates\/.+\/src\/.+\.rs$/.test(f.path));
const registrations = [];
const settingFields = [];
const shortcuts = [];
const uiModules = [];
const catalogs = [];
for (const file of source) {
  const text = git("show", `${baseline}:${file.path}`);
  const lines = text.split("\n");
  if (file.path.includes("/ui/")) uiModules.push(file);
  lines.forEach((line, i) => {
    if (/\bmenu::(?:item\w*|check\w*|toggle|submenu|separator)\s*\(/.test(line)) {
      registrations.push({
        id: `${file.path}:${i + 1}`,
        source: line.trim(),
        context: lines.slice(Math.max(0, i - 2), i + 8).join("\n"),
        status: "retained-source; behavior untested",
      });
    }
    if (/\/(?:settings(?:\/|\.rs)|camera\/settings\.rs)/.test(file.path)) {
      const field = line.match(/^\s*pub(?:\([^)]*\))?\s+([A-Za-z_]\w*)\s*:\s*(.+)/);
      if (field) settingFields.push({ id: `${file.path}:${i + 1}`, name: field[1], type: field[2].trim() });
    }
    if (file.path.includes("/keybinds") || file.path.endsWith("/ui/toolbar.rs")) {
      if (/\b(?:pub )?enum\b|\b(?:Ctrl|Alt|Shift|Escape|F\d+)\b|aurora_i18n::tr!/.test(line)) {
        shortcuts.push({ id: `${file.path}:${i + 1}`, source: line.trim() });
      }
    }
  });
}
for (const file of files.filter((f) => /^crates\/aurora-i18n\/locales\/.*\.ftl$/.test(f.path))) {
  const text = git("show", `${baseline}:${file.path}`);
  catalogs.push({
    ...file,
    keys: [...text.matchAll(/^([A-Za-z][\w-]*)\s*=/gm)].map((match) => match[1]),
  });
}
const inventory = {
  schema: 1,
  upstream: "https://github.com/Aurora-Viewer/Aurora-Viewer",
  baseline,
  reproduce: `git init --bare /tmp/aurora-baseline.git && git --git-dir=/tmp/aurora-baseline.git fetch --depth=1 https://github.com/Aurora-Viewer/Aurora-Viewer.git ${baseline}; node scripts/capture-viewer-inventory.mjs`,
  scope: "All tracked files, all Rust UI modules, menu registration call sites, settings fields, keybinding/toolbar declarations, and locale keys. Dynamic menu choices retain their surrounding source context; this catalog does not prove runtime parity.",
  submodules: git("ls-tree", "-r", baseline).split("\n").filter((line) => line.startsWith("160000 ")),
  lfsFiles: git("lfs", "ls-files", "--name-only", baseline).trim().split("\n").filter(Boolean),
  counts: {
    trackedFiles: files.length,
    rustSourceFiles: source.length,
    uiModules: uiModules.length,
    menuCallSites: registrations.length,
    settingFields: settingFields.length,
    keybindingAndToolbarDeclarations: shortcuts.length,
  },
  files,
  uiModules,
  menuRegistrations: registrations,
  settingFields,
  keybindingAndToolbarDeclarations: shortcuts,
  catalogs,
};
const out = resolve(root, "docs/amante/upstream-inventory.json");
mkdirSync(resolve(root, "docs/amante"), { recursive: true });
writeFileSync(out, `${JSON.stringify(inventory, null, 2)}\n`);
console.log(JSON.stringify({ output: out, ...inventory.counts }, null, 2));
