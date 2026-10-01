// Invisible resize borders: the Overlay has no system frame (decorations off),
// so edges and corners start a native resize. Compact mode resizes only the
// width — its height follows the content.

import { inTauri } from "../ipc/bridge";

type Dir = "North" | "South" | "East" | "West" | "NorthEast" | "NorthWest" | "SouthEast" | "SouthWest";

const EDGES: { dir: Dir; className: string; cursor: string; compact: boolean }[] = [
  { dir: "West", className: "left-0 top-2 bottom-2 w-1.5", cursor: "ew-resize", compact: true },
  { dir: "East", className: "right-0 top-2 bottom-2 w-1.5", cursor: "ew-resize", compact: true },
  { dir: "North", className: "top-0 left-2 right-2 h-1.5", cursor: "ns-resize", compact: false },
  { dir: "South", className: "bottom-0 left-2 right-2 h-1.5", cursor: "ns-resize", compact: false },
  { dir: "NorthWest", className: "left-0 top-0 h-2.5 w-2.5", cursor: "nwse-resize", compact: false },
  { dir: "SouthEast", className: "right-0 bottom-0 h-2.5 w-2.5", cursor: "nwse-resize", compact: false },
  { dir: "NorthEast", className: "right-0 top-0 h-2.5 w-2.5", cursor: "nesw-resize", compact: false },
  { dir: "SouthWest", className: "left-0 bottom-0 h-2.5 w-2.5", cursor: "nesw-resize", compact: false },
];

export function ResizeHandles({ compact }: { compact: boolean }) {
  if (!inTauri()) return null;
  const start = async (dir: Dir, e: React.PointerEvent) => {
    if (e.button !== 0) return;
    e.preventDefault();
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    await getCurrentWindow().startResizeDragging(dir);
  };
  return (
    <>
      {EDGES.filter((h) => !compact || h.compact).map((h) => (
        <div
          key={h.dir}
          aria-hidden
          data-resize={h.dir}
          className={`fixed z-[60] ${h.className}`}
          style={{ cursor: h.cursor }}
          onPointerDown={(e) => void start(h.dir, e)}
        />
      ))}
    </>
  );
}
