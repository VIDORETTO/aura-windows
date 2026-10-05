// Full-screen region selector over a frozen screenshot (004 TK-002).
// Drag to select; Enter or releasing confirms; Esc cancels.

import { useEffect, useRef, useState } from "react";
import { api, errorMessage } from "../ipc/commands";
import { useT } from "../i18n";
import { useApp } from "../state/app";
import { previewSrc } from "./ChipList";
import { DRAFT } from "./session";

export interface Rect {
  x: number;
  y: number;
  w: number;
  h: number;
}

/** Normalizes a drag (any direction) and converts CSS px → image px. */
export function toImageRect(a: { x: number; y: number }, b: { x: number; y: number }, scaleX: number, scaleY: number, maxW: number, maxH: number): Rect {
  const x0 = Math.max(0, Math.min(a.x, b.x)) * scaleX;
  const y0 = Math.max(0, Math.min(a.y, b.y)) * scaleY;
  const x1 = Math.min(maxW, Math.max(a.x, b.x) * scaleX);
  const y1 = Math.min(maxH, Math.max(a.y, b.y) * scaleY);
  return { x: Math.round(x0), y: Math.round(y0), w: Math.max(0, Math.round(x1 - x0)), h: Math.max(0, Math.round(y1 - y0)) };
}

export function parseRegionParams(hash: string) {
  const q = new URLSearchParams(hash.split("?")[1] ?? "");
  return { token: q.get("token") ?? "", path: q.get("path") ?? "", width: Number(q.get("w") ?? 0), height: Number(q.get("h") ?? 0), copy: q.get("mode") === "copy", translate: q.get("mode") === "translate" };
}

export function RegionSelector() {
  const t = useT();
  const { token, path, width, height, copy, translate } = parseRegionParams(window.location.hash);
  const img = useRef<HTMLImageElement>(null);
  const [start, setStart] = useState<{ x: number; y: number } | null>(null);
  const [end, setEnd] = useState<{ x: number; y: number } | null>(null);

  const rect = (): Rect | null => {
    if (!start || !end || !img.current) return null;
    const box = img.current.getBoundingClientRect();
    return toImageRect(start, end, width / (box.width || width), height / (box.height || height), width, height);
  };

  const commit = async () => {
    const r = rect();
    if (!r || r.w < 4 || r.h < 4) return;
    try {
      if (copy) await api.regionCopyText(token, r);
      else if (translate) await api.regionTranslate(token, r, DRAFT);
      else await api.regionCommit(token, r, DRAFT);
    } catch (e) {
      useApp.getState().notify("error", errorMessage(e));
    }
  };

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") void api.regionCancel(token);
      if (e.key === "Enter") void commit();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  const pos = (e: React.PointerEvent) => {
    const box = img.current!.getBoundingClientRect();
    return { x: e.clientX - box.left, y: e.clientY - box.top };
  };
  const sel = start && end ? { left: Math.min(start.x, end.x), top: Math.min(start.y, end.y), width: Math.abs(end.x - start.x), height: Math.abs(end.y - start.y) } : null;

  return (
    <div className="fixed inset-0 cursor-crosshair select-none bg-black" role="application" aria-label={t(copy || translate ? "region.copyHint" : "region.hint")}>
      <img
        ref={img}
        src={previewSrc(path)}
        alt=""
        draggable={false}
        className="h-full w-full object-fill"
        onPointerDown={(e) => {
          (e.target as HTMLElement).setPointerCapture?.(e.pointerId);
          setStart(pos(e));
          setEnd(pos(e));
        }}
        onPointerMove={(e) => start && setEnd(pos(e))}
        onPointerUp={() => void commit()}
      />
      <div className="pointer-events-none absolute inset-0 bg-black/40" style={sel ? { clipPath: `polygon(0 0,100% 0,100% 100%,0 100%,0 0,${sel.left}px ${sel.top}px,${sel.left}px ${sel.top + sel.height}px,${sel.left + sel.width}px ${sel.top + sel.height}px,${sel.left + sel.width}px ${sel.top}px,${sel.left}px ${sel.top}px)` } : undefined} />
      {sel && <div className="pointer-events-none absolute border-2 border-[var(--accent)]" style={sel} />}
      <div className="pointer-events-none absolute left-1/2 top-4 -translate-x-1/2 rounded-full bg-black/70 px-3 py-1 text-[13px] text-white">{t(copy || translate ? "region.copyHint" : "region.hint")}</div>
    </div>
  );
}
