import { accentContrast, clockDuration, exactTime, chordFromEvent, formatBytes, percent } from "./format";

const key = (code: string, mods: Partial<KeyboardEvent> = {}) => ({ code, ctrlKey: false, altKey: false, shiftKey: false, metaKey: false, ...mods }) as KeyboardEvent;

describe("format", () => {
  it("formats bytes and percents", () => {
    expect(formatBytes(0)).toBe("0 B");
    expect(formatBytes(1536)).toBe("1.5 KB");
    expect(formatBytes(670_000_000)).toBe("639 MB");
    expect(percent(50, 200)).toBe(25);
    expect(percent(5, null)).toBe(0);
  });

  it("builds canonical shortcut chords", () => {
    expect(chordFromEvent(key("Space", { ctrlKey: true, shiftKey: true }))).toBe("Ctrl+Shift+Space");
    expect(chordFromEvent(key("KeyA", { altKey: true }))).toBe("Alt+A");
    expect(chordFromEvent(key("F5", { ctrlKey: true }))).toBe("Ctrl+F5");
    expect(chordFromEvent(key("KeyA"))).toBeNull(); // needs a modifier
    expect(chordFromEvent(key("ShiftLeft", { shiftKey: true }))).toBeNull();
  });
});

describe("exactTime", () => {
  it("shows the exact date and time with seconds (QA-031)", () => {
    // 2026-10-03T12:34:56Z
    expect(exactTime(1791030896, "ptBr", "UTC")).toBe("03/10/2026, 12:34:56");
  });
});

describe("clockDuration", () => {
  it("shows recordings as m:ss or h:mm:ss (QA-032)", () => {
    expect(clockDuration(61_400)).toBe("1:01");
    expect(clockDuration(3_723_000)).toBe("1:02:03");
    expect(clockDuration(900)).toBe("0:01");
  });
});

describe("accentContrast", () => {
  it("picks dark text on light accents and white on dark ones (012)", () => {
    expect(accentContrast("#5b5bf7")).toBe("#ffffff");
    expect(accentContrast("#ffd400")).toBe("#1b1b1f");
    // L ≈ 0.235: dark text 4.65:1 vs white 3.68:1.
    expect(accentContrast("#e4572e")).toBe("#1b1b1f");
  });
});
