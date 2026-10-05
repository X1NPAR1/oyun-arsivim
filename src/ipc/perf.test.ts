import { formatGB, formatMB, loadLevel } from "./perf";

describe("performans biçimlendirme", () => {
  it("bellek birimleri Türkçe ondalıkla", () => {
    expect(formatGB(16270 * 1024 ** 2)).toBe("15,9 GB");
    expect(formatMB(1277 * 1024 ** 2)).toBe("1.277 MB");
  });

  it("yük seviyesi eşikleri ve metin etiketi", () => {
    expect(loadLevel(40)).toEqual({ level: "ok", label: "Normal" });
    expect(loadLevel(85)).toEqual({ level: "high", label: "Yüksek" });
    expect(loadLevel(97).level).toBe("critical");
  });
});
