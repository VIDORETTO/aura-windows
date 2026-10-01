// Menus and previews that must never be clipped by the Overlay window: they
// render in a portal, positioned against an anchor, and report how far down
// they reach so the compact window can grow to fit them (useAutoHeight).

import { useLayoutEffect, useRef, useState, type ReactNode, type RefObject } from "react";
import { createPortal } from "react-dom";
import { create } from "zustand";
import { cx } from "./primitives";

interface FloatLayout {
  /** Bottom edge (px, viewport) of each open floating element. */
  bottoms: Record<string, number>;
  report: (id: string, bottom: number | null) => void;
}

export const useFloatLayout = create<FloatLayout>((set) => ({
  bottoms: {},
  report: (id, bottom) =>
    set((s) => {
      const bottoms = { ...s.bottoms };
      if (bottom === null) delete bottoms[id];
      else bottoms[id] = bottom;
      return { bottoms };
    }),
}));

/** Lowest point any floating element needs (0 when none is open). */
export function floatBottom(s: FloatLayout): number {
  return Object.values(s.bottoms).reduce((a, b) => Math.max(a, b), 0);
}

let seq = 0;
const GAP = 6;
const EDGE = 8;

export type Placement = "below" | "above";

/**
 * Positions `children` next to `anchor`. `placement` is a preference: with
 * `grow` (compact window) it always opens below and the window grows; otherwise
 * it flips to the side with more room and scrolls inside the window.
 */
export function Floating({
  anchor,
  placement = "below",
  align = "start",
  grow = false,
  matchWidth = false,
  className,
  children,
}: {
  anchor: RefObject<HTMLElement | null>;
  placement?: Placement;
  align?: "start" | "end";
  grow?: boolean;
  matchWidth?: boolean;
  className?: string;
  children: ReactNode;
}) {
  const [id] = useState(() => `float-${++seq}`);
  const ref = useRef<HTMLDivElement>(null);
  const report = useFloatLayout((s) => s.report);
  const [style, setStyle] = useState<React.CSSProperties>({ visibility: "hidden" });

  useLayoutEffect(() => {
    const place = () => {
      const a = anchor.current?.getBoundingClientRect();
      const el = ref.current;
      if (!a || !el) return;
      const vw = window.innerWidth;
      const vh = window.innerHeight;
      const natural = el.scrollHeight;
      const width = matchWidth ? a.width : Math.min(el.offsetWidth || 0, vw - 2 * EDGE) || undefined;
      const below = vh - a.bottom - GAP - EDGE;
      const above = a.top - GAP - EDGE;
      // Compact window: always below, the window grows. Otherwise the preferred
      // side unless the other one has clearly more room.
      const down = grow || (placement === "below" ? below >= natural || below >= above : !(above >= natural || above >= below));
      const height = grow ? natural : Math.min(natural, Math.max(80, down ? below : above));
      const w = width ?? el.offsetWidth;
      const left = Math.max(EDGE, Math.min(align === "end" ? a.right - w : a.left, vw - EDGE - w));
      const top = down ? a.bottom + GAP : a.top - GAP - height;
      setStyle({ position: "fixed", left, top, width, maxHeight: height, visibility: "visible" });
      report(id, grow ? top + height + EDGE : null);
    };
    place();
    const obs = new ResizeObserver(place);
    if (ref.current) obs.observe(ref.current);
    window.addEventListener("resize", place);
    return () => {
      obs.disconnect();
      window.removeEventListener("resize", place);
      report(id, null);
    };
  }, [anchor, placement, align, grow, matchWidth, id, report]);

  return createPortal(
    <div ref={ref} style={style} className={cx("fade-in z-50 overflow-y-auto", className)}>
      {children}
    </div>,
    document.getElementById("root") ?? document.body,
  );
}
