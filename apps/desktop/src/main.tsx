import { StrictMode, Suspense, lazy } from "react";
import { createRoot } from "react-dom/client";
import "./styles/app.css";
import { OverlayApp } from "./overlay/OverlayApp";
import { RegionSelector } from "./overlay/RegionSelector";
import { bootstrap } from "./state/app";

// Settings is its own chunk: the Overlay (opened on every hotkey) stays small.
const SettingsApp = lazy(() => import("./settings/SettingsApp").then((m) => ({ default: m.SettingsApp })));

function Root() {
  // One bundle, two windows: the hash picks the surface.
  if (window.location.hash.startsWith("#/settings")) {
    return (
      <Suspense fallback={null}>
        <SettingsApp />
      </Suspense>
    );
  }
  if (window.location.hash.startsWith("#/region")) return <RegionSelector />;
  return <OverlayApp />;
}

void bootstrap();
createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <Root />
  </StrictMode>,
);
