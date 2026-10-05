---
name: full-check
description: Oyun Arşivim için tam kalite kapısı — tüm testleri (Vitest, cargo test), linter ve tip kontrolünü (ESLint, tsc strict, clippy -D warnings, cargo fmt, prettier), gizli bilgi taramasını ve isteğe bağlı release build'i çalıştırıp tek bir rapor üretir. "testleri çalıştır", "lint", "build al", "her şey yeşil mi", faz kapanışı veya commit öncesi her doğrulamada kullan.
---

# Tam kontrol (test + lint + build)

Proje kökünden (`D:\OyunArsivim\app`) çalıştır:

```bash
bash .claude/skills/full-check/scripts/full-check.sh          # test + lint
bash .claude/skills/full-check/scripts/full-check.sh --build  # + pnpm tauri build
```

Betik her adımı sırayla çalıştırır, ilk hatada durmaz; sonunda adım adım GEÇTİ/KALDI tablosu basar
ve herhangi biri kaldıysa çıkış kodu 1 olur.

## Rapor
Sonucu kullanıcıya şu biçimde özetle:
- Her adım: GEÇTİ / KALDI (+ test sayıları)
- Kalan adımlar için ilk anlamlı hata satırları ve olası neden
- Build varsa kurulum dosyası boyutları

## Kurallar
- Uyarı da hatadır (`--max-warnings 0`, `-D warnings`). Uyarıyı susturmak için kural kapatma; kök nedeni düzelt.
- Kalan bir adım varsa "bitti" deme. Düzelt, betiği yeniden çalıştır.
- Yeni çözülen önemli hatayı `MEMORY.md` ve `docs/QA.md`'ye yaz.
