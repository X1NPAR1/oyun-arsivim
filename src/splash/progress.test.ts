import { ringGeometry, stepTowards } from "./progress";

describe("açılış ilerlemesi", () => {
  it("gösterilen değer hedefi geçmez ve geri gitmez", () => {
    let v = 0;
    for (let i = 0; i < 200; i++) v = stepTowards(v, 42);
    expect(v).toBe(42);
    expect(stepTowards(50, 30)).toBe(50); // geri gitmez
    expect(stepTowards(10, 10.1)).toBeLessThanOrEqual(10.1);
  });

  it("sayaç sayarak artar (tek karede sıçramaz)", () => {
    expect(stepTowards(0, 100)).toBeLessThan(20);
  });

  it("çember geometrisi üstten başlar ve yüzdeyle orantılı", () => {
    const g = ringGeometry(96, 7, 50);
    expect(g.start).toBeCloseTo(-Math.PI / 2);
    expect(g.end - g.start).toBeCloseTo(Math.PI);
    expect(ringGeometry(96, 7, 150).end - g.start).toBeCloseTo(Math.PI * 2);
  });
});
