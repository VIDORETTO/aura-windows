// Destructive actions ask inside the app: the WebView's window.confirm shows a
// browser dialog ("tauri.localhost says…") that does not belong in a desktop app.

import { useState, type ReactNode } from "react";
import { useT } from "../i18n";
import { Button, cx } from "./primitives";

export function ConfirmButton({
  question,
  onConfirm,
  children,
  label,
  size = "sm",
  variant = "danger",
  className,
  trigger,
}: {
  question: string;
  onConfirm: () => void;
  children?: ReactNode;
  /** Accessible name when the trigger shows only an icon. */
  label?: string;
  size?: "sm" | "md";
  variant?: "danger" | "secondary" | "ghost";
  className?: string;
  /** Custom trigger (e.g. an IconButton); receives the click that asks. */
  trigger?: (ask: () => void) => ReactNode;
}) {
  const t = useT();
  const [asking, setAsking] = useState(false);
  if (asking) {
    return (
      <div role="group" aria-label={question} className={cx("flex flex-wrap items-center gap-1.5", className)}>
        <span className="text-[12px]">{question}</span>
        <Button
          size="sm"
          variant="danger"
          autoFocus
          onClick={() => {
            setAsking(false);
            onConfirm();
          }}
        >
          {t("common.confirm")}
        </Button>
        <Button size="sm" variant="ghost" onClick={() => setAsking(false)}>
          {t("common.cancel")}
        </Button>
      </div>
    );
  }
  if (trigger) return <>{trigger(() => setAsking(true))}</>;
  return (
    <Button size={size} variant={variant} aria-label={label} className={className} onClick={() => setAsking(true)}>
      {children}
    </Button>
  );
}
