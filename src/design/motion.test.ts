import {
  pageVariants,
  scaleDuration,
  staggerDelay,
  STAGGER_MAX_ITEMS,
  STAGGER_STEP,
} from "./motion";

describe("hareket token'ları", () => {
  it("stagger gecikmesi 12. öğeden sonra sabitlenir", () => {
    expect(staggerDelay(0)).toBe(0);
    expect(staggerDelay(3)).toBeCloseTo(3 * STAGGER_STEP);
    expect(staggerDelay(500)).toBeCloseTo(STAGGER_MAX_ITEMS * STAGGER_STEP);
  });

  it("süre seviyeye göre ölçeklenir", () => {
    expect(scaleDuration(0.38, "full")).toBe(0.38);
    expect(scaleDuration(0.38, "reduced")).toBe(0.19);
    expect(scaleDuration(0.38, "off")).toBe(0);
  });

  it("ileri gidişte sağdan, geri dönüşte soldan gelir", () => {
    const v = pageVariants("full");
    expect(v.initial(1).x).toBeGreaterThan(0);
    expect(v.initial(-1).x).toBeLessThan(0);
    expect(v.exit(1).x).toBeLessThan(0);
  });

  it("azaltılmış seviyede kayma ve ölçek yoktur", () => {
    const v = pageVariants("reduced");
    expect(v.initial(1)).toEqual({ opacity: 0, x: 0, scale: 1 });
  });

  it("tüm süreler 150–450 ms aralığındadır", () => {
    const v = pageVariants("full");
    const durations = [v.animate.transition.duration, v.exit(1).transition.duration];
    for (const d of durations) {
      expect(d).toBeGreaterThanOrEqual(0.15);
      expect(d).toBeLessThanOrEqual(0.45);
    }
  });
});
