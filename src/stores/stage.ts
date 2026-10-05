import { create } from "zustand";

/**
 * Pencerenin tamamını kaplayan sinematik sahne görseli. Ana sayfa seçili oyunun görselini buraya
 * yazar; kabuk (AppShell) çizer. Böylece görsel, sayfa geçiş animasyonundaki dönüşümlerden
 * etkilenmeden kenar menüsü ve başlık çubuğunun arkasına kadar uzanır.
 */
export interface StageImage {
  src: string;
  /** Büyük görsel yoksa ikon/kapak bulanık bir renk dokusu olarak kullanılır. */
  ambient: boolean;
}

interface StageState {
  image: StageImage | null;
  setImage: (image: StageImage | null) => void;
}

export const useStageStore = create<StageState>((set) => ({
  image: null,
  setImage: (image) => {
    set({ image });
  },
}));

const decoded = new Map<string, Promise<void>>();

/** Görseli indirip çözer (decode); aynı adres için tek iş yapılır. Hata olursa da çözülür. */
export function preloadImage(src: string): Promise<void> {
  let p = decoded.get(src);
  if (!p) {
    const img = new Image();
    img.decoding = "async";
    img.src = src;
    p = img.decode().catch(() => undefined);
    decoded.set(src, p);
  }
  return p;
}

/** Bellek sınırlı kalsın diye yalnız son kullanılan görsellerin kaydı tutulur. */
export function trimPreloaded(keep: readonly string[]) {
  for (const key of decoded.keys()) {
    if (!keep.includes(key)) decoded.delete(key);
  }
}
