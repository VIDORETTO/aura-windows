import { chordFromEvent, formatBytes, percent } from "./format";

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
