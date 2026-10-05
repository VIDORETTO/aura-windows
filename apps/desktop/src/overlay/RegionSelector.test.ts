import { parseRegionParams, toImageRect } from "./RegionSelector";

describe("region selector", () => {
  it("normalizes drags in any direction and scales to image pixels", () => {
    // Window shown at 50% of the image size (e.g. 200% DPI).
    expect(toImageRect({ x: 100, y: 80 }, { x: 20, y: 10 }, 2, 2, 1920, 1080)).toEqual({ x: 40, y: 20, w: 160, h: 140 });
    // Clamped to the image.
    expect(toImageRect({ x: -5, y: -5 }, { x: 5000, y: 5000 }, 1, 1, 1920, 1080)).toEqual({ x: 0, y: 0, w: 1920, h: 1080 });
  });

  it("reads the frozen screen from the route", () => {
    const p = parseRegionParams("#/region?token=abc&path=C%3A%2FUsers%2Fa%20b%2Ff.png&w=1920&h=1080");
    expect(p).toEqual({ token: "abc", path: "C:/Users/a b/f.png", width: 1920, height: 1080, copy: false });
    expect(parseRegionParams("#/region?token=abc&path=x&w=10&h=10&mode=copy").copy).toBe(true);
  });
});
