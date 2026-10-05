import { relativeOffset, wheelStep, wrapIndex } from "./carousel";

describe("logo şeridi gezinmesi", () => {
  it("indeks iki yönde de döngüye girer", () => {
    expect(wrapIndex(5, 5)).toBe(0);
    expect(wrapIndex(-1, 5)).toBe(4);
    expect(wrapIndex(3, 0)).toBe(0);
  });

  it("göreli konum en kısa yolu seçer", () => {
    expect(relativeOffset(0, 0, 5)).toBe(0);
    expect(relativeOffset(1, 0, 5)).toBe(1);
    expect(relativeOffset(4, 0, 5)).toBe(-1); // son öğe ilk öğenin solunda
    expect(relativeOffset(0, 4, 5)).toBe(1); // döngüde ilk öğe sondan sonra gelir
    expect(relativeOffset(2, 0, 4)).toBe(-2);
  });

  it("tekerlek birikir ve eşikte tek adım atar", () => {
    expect(wheelStep(0, 30)).toEqual([0, 30]);
    expect(wheelStep(30, 40)).toEqual([1, 0]);
    expect(wheelStep(0, -120)).toEqual([-1, 0]);
    expect(wheelStep(-50, 20)).toEqual([0, -30]);
  });
});
