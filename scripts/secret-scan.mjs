#!/usr/bin/env node
// Commit öncesi ve CI'da gizli bilgi taraması. aitmpl "secret-scanner" hook'unun Node portu
// (bu makinede gerçek Python yok). Kullanım: `node scripts/secret-scan.mjs [--staged | --all]`.
import { execFileSync } from "node:child_process";
import { readFileSync, existsSync } from "node:fs";

const PATTERNS = [
  [/AKIA[0-9A-Z]{16}/, "AWS Access Key ID"],
  [/sk-ant-api\d{2}-[A-Za-z0-9_-]{20,}/, "Anthropic API anahtarı"],
  [/sk-(proj-)?[A-Za-z0-9_-]{32,}/, "OpenAI API anahtarı"],
  [/AIza[0-9A-Za-z_-]{35}/, "Google API anahtarı"],
  [/gh[pousr]_[A-Za-z0-9]{36,}/, "GitHub token"],
  [/github_pat_[A-Za-z0-9_]{60,}/, "GitHub ince taneli token"],
  [/-----BEGIN (RSA |EC |OPENSSH |ENCRYPTED )?PRIVATE KEY-----/, "Özel anahtar"],
  [/untrusted comment: (rsign|minisign) encrypted secret key/, "Tauri updater gizli anahtarı"],
  [
    /(?:steamgriddb|sgdb)[_-]?(?:api)?[_-]?key\s*[:=]\s*["'][a-f0-9]{32}["']/i,
    "SteamGridDB anahtarı",
  ],
];
const FORBIDDEN_FILES = [/(^|\/)\.env(\.|$)/, /\.key$/, /\.pfx$/, /\.p12$/];
const SELF = "scripts/secret-scan.mjs";

const mode = process.argv[2] ?? "--staged";
const args =
  mode === "--all" ? ["ls-files"] : ["diff", "--cached", "--name-only", "--diff-filter=ACM"];
const files = execFileSync("git", args, { encoding: "utf8" }).split("\n").filter(Boolean);

const findings = [];
for (const file of files) {
  if (file === SELF) continue;
  if (FORBIDDEN_FILES.some((re) => re.test(file)) && !file.endsWith(".env.example")) {
    findings.push(`${file}: bu dosya türü depoya girmemeli`);
    continue;
  }
  if (!existsSync(file)) continue;
  const buf = readFileSync(file);
  if (buf.includes(0)) continue; // ikili dosya
  const text = buf.toString("utf8");
  text.split(/\r?\n/).forEach((line, i) => {
    for (const [re, label] of PATTERNS)
      if (re.test(line)) findings.push(`${file}:${i + 1}: ${label}`);
  });
}

if (findings.length) {
  console.error("Gizli bilgi bulundu, işlem durduruldu:\n  " + findings.join("\n  "));
  process.exit(1);
}
console.log(`Gizli bilgi taraması temiz (${files.length} dosya).`);
