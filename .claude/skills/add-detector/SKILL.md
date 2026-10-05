---
name: add-detector
description: Oyun Arşivim'e yeni bir oyun platformu dedektörü ekler — Rust GameDetector modül şablonu, sahte (fixture) dosyalar, birim testleri, kayıt ve docs/PLATFORMS.md girdisi. "X platformunu destekle", "yeni dedektör", "Y launcher'ındaki oyunları bul" denince kullan.
---

# Yeni platform dedektörü ekleme

`<id>` = küçük harfli modül adı (ör. `gog`), `<Name>` = PascalCase (ör. `Gog`).

1. **Araştır, varsayma:** Platformun kurulu oyunları nerede tuttuğunu (kayıt defteri anahtarı, manifest
   klasörü, veritabanı) ve başlatma yöntemini (protokol URL'si / exe) doğrula. Bu makinede gerçek örnek
   varsa formatını incele (kişisel veri kopyalama). Bulguları önce `docs/PLATFORMS.md`'ye yaz.
2. **Modül:** `assets/detector_template.rs` dosyasını `src-tauri/src/detectors/<id>.rs` olarak kopyala,
   `__ID__` / `__NAME__` yer tutucularını değiştir. Kök yol **enjekte edilebilir** olmalı ki testler sahte
   klasörle çalışsın — gerçek sistem yolu yalnızca `system_default()` içinde çözülür.
3. **Kayıt:** `src-tauri/src/detectors/mod.rs` içinde `pub mod <id>;` ve `all_detectors()` listesine ekle.
4. **Fixture:** `src-tauri/tests/fixtures/<id>/` altına gerçek formattan türetilmiş anonim örnekler:
   en az 1 geçerli oyun, 1 araç/redistributable (filtrelenmeli), 1 bozuk dosya, Türkçe yol (`Şahin'in Oyunu`).
5. **Testler (zorunlu):** geçerli oyunlar bulunur · filtrelenecekler elenir · bozuk dosya paniğe yol
   açmaz, atlanır ve loglanır · kök yok → boş liste (hata değil) · Türkçe yol · başlatma hedefi doğru.
6. **Doğrula:** `full-check` skill'i; ardından uygulamada gerçek tarama yapıp bulunan oyun sayısını
   (isimsiz) `docs/QA.md`'ye yaz. Sınırlamaları `docs/PLATFORMS.md`'ye ekle.
