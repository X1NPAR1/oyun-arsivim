// package.json, tauri.conf.json ve Cargo.toml'daki sürümü tek komutla eşitler.
import { readFileSync, writeFileSync } from "node:fs";

const version = process.argv[2];
if (!version || !/^\d+\.\d+\.\d+$/.test(version)) {
  console.error("Kullanım: node scripts/set-version.mjs 0.3.0");
  process.exit(1);
}
for (const file of ["package.json", "src-tauri/tauri.conf.json"]) {
  const json = JSON.parse(readFileSync(file, "utf8"));
  json.version = version;
  writeFileSync(file, JSON.stringify(json, null, 2) + "\n");
}
const cargo = "src-tauri/Cargo.toml";
writeFileSync(
  cargo,
  readFileSync(cargo, "utf8").replace(/^version = ".*"$/m, `version = "${version}"`),
);
