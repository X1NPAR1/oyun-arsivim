#!/usr/bin/env bash
# Oyun Arşivim tam kalite kapısı. Proje kökünden çalıştırılır.
set -u
. "$(dirname "$0")/../../../../scripts/dev-env.sh"
BUILD=0
[ "${1:-}" = "--build" ] && BUILD=1
NAMES=()
RESULTS=()

run() {
  local name="$1"
  shift
  echo "::: $name"
  if "$@"; then RESULTS+=("GEÇTİ"); else RESULTS+=("KALDI"); fi
  NAMES+=("$name")
}

run "TypeScript (strict)" pnpm -s typecheck
run "ESLint" pnpm -s lint
run "Prettier" pnpm -s format:check
run "Vitest" pnpm -s test
run "cargo fmt" cargo fmt --manifest-path src-tauri/Cargo.toml --check
run "cargo clippy" cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
run "cargo test" cargo test --manifest-path src-tauri/Cargo.toml
run "Gizli bilgi taraması" pnpm -s secrets
if [ "$BUILD" = 1 ]; then run "Release build" pnpm tauri build; fi

echo
echo "================ RAPOR ================"
FAIL=0
for i in "${!NAMES[@]}"; do
  printf '%-24s %s\n' "${NAMES[$i]}" "${RESULTS[$i]}"
  [ "${RESULTS[$i]}" = "KALDI" ] && FAIL=1
done
if [ "$BUILD" = 1 ]; then
  ls -la src-tauri/target/release/bundle/nsis/*.exe src-tauri/target/release/bundle/msi/*.msi 2>/dev/null
fi
exit $FAIL
