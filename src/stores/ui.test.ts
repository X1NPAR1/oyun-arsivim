import { resolveMotionLevel, useUiStore } from "./ui";

describe("animasyon seviyesi", () => {
  it("sistem tercihi Windows ayarını izler", () => {
    expect(resolveMotionLevel("system", false)).toBe("full");
    expect(resolveMotionLevel("system", true)).toBe("reduced");
  });

  it("açık kullanıcı tercihi sistem ayarını ezer", () => {
    expect(resolveMotionLevel("full", true)).toBe("full");
    expect(resolveMotionLevel("off", false)).toBe("off");
  });

  it("tercih kalıcı olarak saklanır", () => {
    useUiStore.getState().setMotionPreference("reduced");
    expect(localStorage.getItem("oa.motionPreference")).toBe("reduced");
    expect(useUiStore.getState().motionPreference).toBe("reduced");
  });
});
