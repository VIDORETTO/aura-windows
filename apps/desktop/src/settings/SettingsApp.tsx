import { AppWindow, Bot, Info, Keyboard, KeyRound, Mic, Puzzle, Settings2, Shield, Stethoscope } from "lucide-react";
import { ProfilesSection } from "./Profiles";
import { useEffect, useRef, useState, type ComponentType } from "react";
import { inTauri } from "../ipc/bridge";
import { useT, type MessageKey } from "../i18n";
import { Toasts } from "../ui/Toasts";
import { SpeechConsentDialog } from "../ui/SpeechConsentDialog";
import { cx } from "../ui/primitives";
import { AboutSection, DiagnosticsSection } from "./Diagnostics";
import { AccountSection } from "./Account";
import { ExtensionsSection } from "./Extensions";
import { GeneralSection } from "./General";
import { PrivacySection } from "./Privacy";
import { ProvidersSection } from "./Providers";
import { ShortcutsSection } from "./Shortcuts";
import { VoiceSection } from "./Voice";
import { SettingsSearch, revealText } from "./SettingsSearch";

interface Page {
  id: string;
  label: MessageKey;
  icon: ComponentType<{ size?: number }>;
  Component: ComponentType;
}

export const PAGES: Page[] = [
  { id: "general", label: "settings.general", icon: Settings2, Component: GeneralSection },
  { id: "account", label: "settings.account", icon: Bot, Component: AccountSection },
  { id: "providers", label: "settings.providers", icon: KeyRound, Component: ProvidersSection },
  { id: "privacy", label: "settings.privacy", icon: Shield, Component: PrivacySection },
  { id: "voice", label: "settings.voice", icon: Mic, Component: VoiceSection },
  { id: "extensions", label: "settings.extensions", icon: Puzzle, Component: ExtensionsSection },
  { id: "profiles", label: "settings.profiles", icon: AppWindow, Component: ProfilesSection },
  { id: "shortcuts", label: "settings.shortcuts", icon: Keyboard, Component: ShortcutsSection },
  { id: "diagnostics", label: "settings.diagnostics", icon: Stethoscope, Component: DiagnosticsSection },
  { id: "about", label: "settings.about", icon: Info, Component: AboutSection },
];

export function pageFromHash(hash: string): string {
  const id = hash.replace(/^#\/settings\/?/, "").split(/[/?]/)[0];
  return PAGES.some((p) => p.id === id) ? id : "general";
}

export function SettingsApp() {
  const t = useT();
  const [page, setPage] = useState(() => pageFromHash(window.location.hash));
  const main = useRef<HTMLElement>(null);
  const search = useRef<HTMLInputElement>(null);
  /** Text to flash once the page chosen in the search has rendered. */
  const [reveal, setReveal] = useState<{ text: string; n: number } | null>(null);
  // Each page opens at its top (it used to keep the previous page's scroll).
  useEffect(() => {
    if (main.current) main.current.scrollTop = 0;
  }, [page]);

  useEffect(() => {
    if (!reveal || !main.current) return;
    const root = main.current;
    const h = setTimeout(() => revealText(root, reveal.text), 60);
    return () => clearTimeout(h);
  }, [reveal, page]);

  // Ctrl+K / Ctrl+F: search the settings (017).
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && !e.shiftKey && !e.altKey && (e.key.toLowerCase() === "k" || e.key.toLowerCase() === "f")) {
        e.preventDefault();
        search.current?.focus();
        search.current?.select();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  useEffect(() => {
    document.body.classList.add("settings");
    const onHash = () => setPage(pageFromHash(window.location.hash));
    window.addEventListener("hashchange", onHash);
    let off: (() => void) | undefined;
    if (inTauri()) {
      void import("@tauri-apps/api/event").then(async ({ listen }) => {
        off = await listen<string>("aura://navigate", (e) => {
          window.location.hash = e.payload;
        });
      });
    }
    return () => {
      window.removeEventListener("hashchange", onHash);
      off?.();
    };
  }, []);

  const Current = PAGES.find((p) => p.id === page)!.Component;
  return (
    <div className="flex h-full">
      <nav aria-label={t("settings.title")} className="flex w-56 shrink-0 flex-col gap-0.5 overflow-y-auto border-r border-line p-2">
        <h1 className="px-2 pb-2 pt-1 text-sm font-semibold">{t("settings.title")}</h1>
        <div className="pb-2">
          <SettingsSearch
            pages={PAGES}
            inputRef={search}
            onOpen={(id, text) => {
              window.location.hash = `#/settings/${id}`;
              setPage(id);
              setReveal((r) => ({ text, n: (r?.n ?? 0) + 1 }));
            }}
          />
        </div>
        {PAGES.map((p) => (
          <a
            key={p.id}
            href={`#/settings/${p.id}`}
            aria-current={page === p.id ? "page" : undefined}
            className={cx("flex items-center gap-2 rounded-md px-2 py-1.5 text-[13px] hover:bg-hover", page === p.id && "bg-hover font-medium text-accent")}
          >
            <p.icon size={15} /> {t(p.label)}
          </a>
        ))}
      </nav>
      <main ref={main} className="min-w-0 flex-1 overflow-y-auto">
        <div className="mx-auto flex max-w-3xl flex-col gap-4 p-6">
          <Current />
        </div>
      </main>
      <Toasts />
      <SpeechConsentDialog />
    </div>
  );
}
