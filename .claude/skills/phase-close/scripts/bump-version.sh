#!/usr/bin/env bash
# Sürümü tüm manifestlerde günceller. Kullanım: bump-version.sh 0.3.0
set -euo pipefail
V="$1"
[[ "$V" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "Geçersiz sürüm: $V"; exit 1; }
node scripts/set-version.mjs "$V"
. scripts/dev-env.sh
cargo update --manifest-path src-tauri/Cargo.toml -p oyun-arsivim --offline > /dev/null
npx prettier --write package.json src-tauri/tauri.conf.json > /dev/null
echo "Sürüm $V olarak ayarlandı."
