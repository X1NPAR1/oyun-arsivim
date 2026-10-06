import { dominantColor, rgbToHsl, toAccent } from "./color";

function pixels(colors: [number, number, number][], count: number): Uint8ClampedArray {
  const out = new Uint8ClampedArray(colors.length * count * 4);
  let i = 0;
  for (const [r, g, b] of colors) {
    for (let k = 0; k < count; k++) {
      out[i++] = r;
      out[i++] = g;
      out[i++] = b;
      out[i++] = 255;
    }
  }
  return out;
}

describe("dinamik renk", () => {
  it("HSL dönüşümü", () => {
    const [h, s, l] = rgbToHsl({ r: 255, g: 0, b: 0 });
    expect([Math.round(h), s, l]).toEqual([0, 1, 0.5]);
  });

  it("baskın tonu bulur; gri, siyah ve beyazı yok sayar", () => {
    const px = pixels(
      [
        [230, 120, 20], // turuncu (çoğunluk)
        [20, 20, 20], // siyah
        [128, 128, 128], // gri
        [30, 80, 200], // mavi (azınlık)
      ],
      40,
    );
    const extra = pixels([[235, 125, 25]], 60);
    const all = new Uint8ClampedArray(px.length + extra.length);
    all.set(px);
    all.set(extra, px.length);
    const c = dominantColor(all, 1);
    expect(c).not.toBeNull();
    const [h] = rgbToHsl(c ?? { r: 0, g: 0, b: 0 });
    expect(h).toBeGreaterThan(20);
    expect(h).toBeLessThan(40);
  });

  it("uygun renk yoksa null", () => {
    expect(
      dominantColor(
        pixels(
          [
            [10, 10, 10],
            [250, 250, 250],
          ],
          50,
        ),
        1,
      ),
    ).toBeNull();
  });

  it("vurgu rengi okunur aralığa kıstırılır", () => {
    expect(toAccent({ r: 255, g: 0, b: 0 })).toBe("hsl(0 65% 55%)");
    expect(toAccent({ r: 60, g: 40, b: 30 })).toMatch(/^hsl\(\d+ 35% 55%\)$/);
  });
});

describe("logo parlaklığı", () => {
  it("opak piksellerin ortalamasını alır, saydamları yok sayar", async () => {
    const { averageLuminance, DARK_LOGO_THRESHOLD } = await import("./color");
    const dark = new Uint8ClampedArray([20, 30, 70, 255, 255, 255, 255, 0]); // lacivert + saydam beyaz
    expect(averageLuminance(dark)).toBeLessThan(DARK_LOGO_THRESHOLD);
    const light = new Uint8ClampedArray([240, 240, 240, 255]);
    expect(averageLuminance(light)).toBeGreaterThan(0.9);
    expect(averageLuminance(new Uint8ClampedArray([0, 0, 0, 0]))).toBeNull();
  });
});

describe("karma logo", () => {
  it("çoğunluğu koyu olan logo koyu sayılır (beyaz bir parça olsa bile)", async () => {
    const { darkPixelShare } = await import("./color");
    const px: number[] = [];
    for (let i = 0; i < 6; i++) px.push(20, 30, 80, 255); // lacivert yazı
    for (let i = 0; i < 4; i++) px.push(250, 250, 250, 255); // beyaz "2"
    expect(darkPixelShare(new Uint8ClampedArray(px))).toBeCloseTo(0.6);
  });
});
