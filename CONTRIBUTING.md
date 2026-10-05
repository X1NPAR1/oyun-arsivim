# Katkı Rehberi

1. `main` her zaman çalışır durumdadır. İşler `faz/NN-kisa-ad` veya `fix/kisa-ad` dalında yapılır.
2. Commit mesajları [Conventional Commits](https://www.conventionalcommits.org/tr/): `feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `chore:`.
3. Göndermeden önce `pnpm check` tamamen yeşil olmalı (uyarı da hatadır).
4. Hata düzeltmesi önce hatayı yeniden üreten bir testle başlar.
5. Yeni bağımlılık, font, ikon veya görsel → `docs/THIRD_PARTY.md`'ye lisansıyla eklenir.
6. API anahtarı, token, sertifika veya kişisel veri asla depoya girmez (`pnpm secrets`).
7. Arayüz kuralları: SVG yok, sistem fontu yok, sert köşeli kutu yok — bkz. `docs/DESIGN_SYSTEM.md`.
