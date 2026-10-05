---
name: phase-close
description: Oyun Arşivim faz kapanış protokolünü (prompt Bölüm 4.2) uçtan uca uygular — tam kontrol, gerçek kullanım testi ve ekran görüntüsü, doküman güncelleme, Conventional Commit, faz dalını main'e birleştirme, sürüm etiketi, GitHub Release ve CI doğrulaması. Bir faz bittiğinde, "fazı kapat", "release çıkar", "sürüm yayınla" denince kullan.
---

# Faz kapanış protokolü

Değişkenler: `NN` = iki haneli faz no (ör. 03), `X` = faz no, dal = `faz/NN-kisa-ad`,
etiket = `v0.X.0` (Faz 18 → `v1.0.0`).

1. **Tam kontrol:** `bash .claude/skills/full-check/scripts/full-check.sh --build` → hepsi GEÇTİ.
2. **Dev modu:** `pnpm tauri dev` arka planda başlat, fazın özelliklerini gerçekten kullan, kapat.
3. **Release:** `design-review` skill'i ile `-Launch` kullanarak ekran görüntüleri al → `review/faz-NN/`, değerlendir.
4. **Kabul kriterleri:** `docs/PLAN.md` içindeki faz kriterlerini tek tek doğrula; eksik varsa düzelt → 1.
5. **Dokümanlar:** `docs/PROGRESS.md`, `MEMORY.md`, `CHANGELOG.md` (`## [0.X.0] - YYYY-MM-DD`),
   gerekirse `README.md`, `docs/QA.md`.
6. **Sürüm:** `bash .claude/skills/phase-close/scripts/bump-version.sh 0.X.0`
7. **Commit + merge + tag + push + CI bekleme:**
   `bash .claude/skills/phase-close/scripts/release.sh NN kisa-ad 0.X.0 "feat: faz X — <özet>"`
8. **CI kırmızıysa:** `gh-fix-ci` skill'i ile logları incele, düzelt, `fix:` commit'iyle tekrar gönder.
   Yeşil olmadan sonraki faza geçme.
9. **Release notları:** `gh release edit v0.X.0 --notes-file <CHANGELOG bölümü>`, ekran görüntülerini
   `gh release upload` ile ekle. Faz < 18 ise pre-release.
10. **Rapor (3–5 satır):** ne yapıldı, test sonuçları, release bağlantısı.
11. Onay beklemeden sonraki fazın dalını aç: `git switch -c faz/NN-ad main`.
