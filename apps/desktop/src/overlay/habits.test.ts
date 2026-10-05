// 048: habits only count command names per app, and only after three uses.
import { clearHabits, recordHabit, topHabits } from "./habits";

describe("habits", () => {
  beforeEach(() => clearHabits());

  it("a command becomes a habit after three uses in the same app", () => {
    recordHabit("Chrome.exe", "traduzir");
    recordHabit("chrome.exe", "traduzir");
    expect(topHabits("chrome.exe")).toEqual([]);
    recordHabit("chrome.exe", "traduzir");
    expect(topHabits("CHROME.EXE")).toEqual(["traduzir"]);
    expect(topHabits("code.exe")).toEqual([]);
  });

  it("orders by use, limits the list and can be cleared", () => {
    for (let i = 0; i < 4; i++) recordHabit("a.exe", "tldr");
    for (let i = 0; i < 5; i++) recordHabit("a.exe", "golpe");
    for (let i = 0; i < 3; i++) recordHabit("a.exe", "formal");
    expect(topHabits("a.exe")).toEqual(["golpe", "tldr"]);
    clearHabits();
    expect(topHabits("a.exe")).toEqual([]);
  });

  it("ignores an unknown app", () => {
    recordHabit(null, "tldr");
    recordHabit("", "tldr");
    expect(localStorage.getItem("aura.habits")).toBeNull();
  });
});
