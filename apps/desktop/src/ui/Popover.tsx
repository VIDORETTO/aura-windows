// Button-anchored popover (menus, appearance) on top of <Floating>: closes on
// outside click and Esc (which never reaches the Overlay's own Esc handling).

import { useEffect, useId, useRef, useState, type ReactNode } from "react";
import { Floating } from "./floating";
import { cx } from "./primitives";

export function usePopover() {
  const [open, setOpen] = useState(false);
  const anchor = useRef<HTMLButtonElement>(null);
  const panel = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!open) return;
    const onDown = (e: PointerEvent) => {
      const t = e.target as Node;
      if (!anchor.current?.contains(t) && !panel.current?.contains(t)) setOpen(false);
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.stopPropagation();
        setOpen(false);
        anchor.current?.focus();
      }
    };
    document.addEventListener("pointerdown", onDown, true);
    document.addEventListener("keydown", onKey, true);
    return () => {
      document.removeEventListener("pointerdown", onDown, true);
      document.removeEventListener("keydown", onKey, true);
    };
  }, [open]);
  return { open, setOpen, toggle: () => setOpen((o) => !o), anchor, panel };
}

export function PopoverPanel({
  pop,
  label,
  align = "start",
  grow,
  className,
  children,
}: {
  pop: ReturnType<typeof usePopover>;
  label: string;
  align?: "start" | "end";
  grow?: boolean;
  className?: string;
  children: ReactNode;
}) {
  if (!pop.open) return null;
  return (
    <Floating anchor={pop.anchor} align={align} grow={grow}>
      <div ref={pop.panel} role="dialog" aria-label={label} className={cx("menu-surface rounded-md border border-line p-1.5 shadow-lg", className)}>
        {children}
      </div>
    </Floating>
  );
}

/** One selectable row inside a popover menu. */
export function MenuOption({ selected, onSelect, children, hint }: { selected?: boolean; onSelect: () => void; children: ReactNode; hint?: string }) {
  const id = useId();
  return (
    <button
      type="button"
      role="menuitemradio"
      aria-checked={selected}
      aria-labelledby={`${id}-label`}
      aria-describedby={hint ? `${id}-hint` : undefined}
      onClick={onSelect}
      className={cx(
        "flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-[13px] hover:bg-hover",
        selected && "text-accent",
      )}
    >
      <span className={cx("h-1.5 w-1.5 shrink-0 self-start rounded-full mt-[7px]", selected ? "bg-accent" : "bg-transparent")} aria-hidden />
      <span className="min-w-0 flex-1">
        <span id={`${id}-label`} className="block truncate">{children}</span>
        {hint && <span id={`${id}-hint`} className="block text-[11px] leading-snug text-muted">{hint}</span>}
      </span>
    </button>
  );
}

export function MenuLabel({ children }: { children: ReactNode }) {
  return <div className="px-2 pb-0.5 pt-1.5 text-[11px] font-medium uppercase tracking-wide text-muted">{children}</div>;
}
