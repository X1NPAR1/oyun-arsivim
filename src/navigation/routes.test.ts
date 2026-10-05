import { navDirection, NAV_ITEMS } from "./routes";

describe("navDirection", () => {
  it("sidebar'da aşağı gitmek ileri, yukarı gitmek geridir", () => {
    expect(navDirection("/", "/kutuphane")).toBe(1);
    expect(navDirection("/ayarlar", "/kutuphane")).toBe(-1);
  });

  it("detaya inmek ileri, detaydan dönmek geridir", () => {
    expect(navDirection("/kutuphane", "/kutuphane/42")).toBe(1);
    expect(navDirection("/kutuphane/42", "/kutuphane")).toBe(-1);
  });

  it("detaydan başka bir üst sayfaya geçiş sidebar sırasını izler", () => {
    expect(navDirection("/kutuphane/42", "/")).toBe(-1);
    expect(navDirection("/kutuphane/42", "/ayarlar")).toBe(1);
  });

  it("her menü öğesinin benzersiz yolu ve Türkçe etiketi var", () => {
    const paths = new Set(NAV_ITEMS.map((i) => i.path));
    expect(paths.size).toBe(NAV_ITEMS.length);
    expect(NAV_ITEMS.map((i) => i.label)).toContain("Kütüphane");
  });
});
