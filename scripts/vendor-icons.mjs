// Phosphor ikon fontundan yalnızca woff2 dosyalarını ve kullanılan ikonların kod noktalarını alır.
// Neden: paketin CSS'i yedek olarak SVG font (Phosphor.svg) referans veriyor — projede SVG yasak.
// Yeni ikon gerekince ICONS listesine ekle ve `pnpm icons` çalıştır.
import { copyFileSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";

const ICONS = [
  "house",
  "squares-four",
  "chart-line-up",
  "cloud-arrow-up",
  "gear-six",
  "magnifying-glass",
  "minus",
  "square",
  "copy",
  "x",
  "play",
  "game-controller",
  "sparkle",
  "clock",
  "star",
  "arrow-left",
  "arrow-right",
  "caret-right",
  "check",
  "warning",
  "info",
  "folder-open",
  "plus",
  "trash",
  "list",
  "lightning",
  "hard-drives",
  "heart",
  "shuffle",
  "command",
  "arrows-clockwise",
  "file-text",
  "download-simple",
  "monitor",
  "prohibit",
  "tray",
  "power",
  "cpu",
  "memory",
  "gauge",
  "rocket-launch",
  "broom",
  "toggle-right",
];

const pkg = "node_modules/@phosphor-icons/web/src";
const outFonts = "src/assets/fonts";
mkdirSync(outFonts, { recursive: true });

const variants = [
  ["regular", "Phosphor", "style.css", ".ph-"],
  ["fill", "Phosphor-Fill", "style.css", ".ph-fill.ph-"],
];

const codes = {};
for (const [dir, file, css, prefix] of variants) {
  copyFileSync(`${pkg}/${dir}/${file}.woff2`, `${outFonts}/${file}.woff2`);
  const text = readFileSync(`${pkg}/${dir}/${css}`, "utf8");
  for (const name of ICONS) {
    const escaped = (prefix + name).replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    const match = new RegExp(`${escaped}:before\\s*\\{\\s*content:\\s*"\\\\([0-9a-f]+)"`).exec(
      text,
    );
    if (!match) throw new Error(`İkon bulunamadı: ${dir}/${name}`);
    codes[name] ??= {};
    codes[name][dir] = parseInt(match[1], 16);
  }
}

const body = Object.entries(codes)
  .map(
    ([name, v]) =>
      `  "${name}": { regular: 0x${v.regular.toString(16)}, fill: 0x${v.fill.toString(16)} },`,
  )
  .join("\n");
writeFileSync(
  "src/design/iconCodes.ts",
  `// Bu dosya scripts/vendor-icons.mjs tarafından üretilir. Elle düzenleme.\n` +
    `export const ICON_CODES = {\n${body}\n} as const;\n\nexport type IconName = keyof typeof ICON_CODES;\n`,
);
console.log(`${ICONS.length} ikon üretildi.`);
