---
name: design-review
description: Oyun Arşivim'in gerçek penceresinin ekran görüntüsünü alır (review/faz-XX/) ve docs/DESIGN_SYSTEM.md kurallarına göre maddeli değerlendirme yapar. Arayüz değişikliğinden sonra, faz kapanışında, "ekran görüntüsü al", "tasarımı kontrol et", "nasıl görünüyor" denince kullan.
---

# Ekran görüntüsü + tasarım değerlendirmesi

## 1. Görüntü al
Uygulama açıkken (dev veya release) PowerShell ile:

```powershell
powershell -ExecutionPolicy Bypass -File .claude/skills/design-review/scripts/capture-window.ps1 -Phase 02 -Name kutuphane
# Uygulamayı kendisi başlatıp kapatması için: -Launch (release exe'yi kullanır)
```

Çıktı: `review/faz-XX/<ad>.png`. Görseli Read aracıyla aç ve gerçekten incele.
Animasyon kontrolü için `-Burst 6` ile ~80 ms aralıklı kare dizisi alınabilir.

## 2. Değerlendirme kontrol listesi (her madde GEÇTİ/KALDI + not)
1. **Sert dikdörtgen yok:** tüm yüzeylerde 16–28px yarıçap; çizgi ayırıcı yok.
2. **Cam/derinlik:** paneller blur + yarı saydam + ince ışık kenarı veya kapak rengi degradesi.
3. **Tipografi:** başlık Sora, metin Manrope; Türkçe karakterler (ç ğ ı İ ö ş ü) doğru; varsayılan font yok.
4. **İkonlar:** ikon fontu/PNG; SVG yok (`rg -i "\.svg|<svg" src` boş olmalı).
5. **Renk/kontrast:** koyu tema, vurgu rengi tutarlı, metin kontrastı ≥ 4.5:1.
6. **Hiyerarşi ve boşluk:** 4/8px ızgarası, taşma/kesilme yok, uzun Türkçe metinler sığıyor.
7. **Yüzen sidebar ve özel başlık çubuğu** (Faz 2+).
8. **Hareket:** geçişler 150–450 ms, yön mantıklı, takılma yok (burst kareleri).

Sonucu `docs/QA.md` → "Tasarım değerlendirmeleri" bölümüne faz numarasıyla yaz. KALDI varsa düzelt ve tekrar görüntü al.
