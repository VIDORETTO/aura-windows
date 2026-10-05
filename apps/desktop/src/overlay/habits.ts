// Habits (048): which quick command the user runs most per app, so the empty
// Overlay offers it first. Counts live only in this browser profile (local),
// hold command names (never text) and can be cleared.

const KEY = "aura.habits";
/** Uses before a command counts as a habit. */
export const HABIT_MIN = 3;

type Counts = Record<string, Record<string, number>>;

function read(): Counts {
  try {
    const raw = JSON.parse(localStorage.getItem(KEY) ?? "{}");
    return raw && typeof raw === "object" ? (raw as Counts) : {};
  } catch {
    return {};
  }
}

export function recordHabit(processName: string | null | undefined, command: string): void {
  const app = (processName ?? "").toLowerCase();
  if (!app || !command) return;
  try {
    const all = read();
    const mine = (all[app] ??= {});
    mine[command] = (mine[command] ?? 0) + 1;
    localStorage.setItem(KEY, JSON.stringify(all));
  } catch {
    /* storage unavailable: habits are only a convenience */
  }
}

/** Commands used at least `HABIT_MIN` times in `processName`, most used first. */
export function topHabits(processName: string | null | undefined, limit = 2): string[] {
  const mine = read()[(processName ?? "").toLowerCase()] ?? {};
  return Object.entries(mine)
    .filter(([, n]) => n >= HABIT_MIN)
    .sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]))
    .slice(0, limit)
    .map(([c]) => c);
}

export function clearHabits(): void {
  try {
    localStorage.removeItem(KEY);
  } catch {
    /* ignore */
  }
}
