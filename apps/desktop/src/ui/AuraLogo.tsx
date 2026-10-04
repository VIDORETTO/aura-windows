// Aura's logo in the current accent color (012): same shapes as public/logo.svg.
export function AuraLogo({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 512 512" className={className} aria-hidden="true">
      <defs>
        <linearGradient id="aura-logo-bg" x1="0" y1="0" x2="1" y2="1">
          <stop offset="0" style={{ stopColor: "var(--accent)" }} />
          <stop offset="1" style={{ stopColor: "color-mix(in srgb, var(--accent) 65%, #ffffff)" }} />
        </linearGradient>
        <radialGradient id="aura-logo-glow" cx="0.5" cy="0.42" r="0.6">
          <stop offset="0" stopColor="#ffffff" stopOpacity="0.35" />
          <stop offset="1" stopColor="#ffffff" stopOpacity="0" />
        </radialGradient>
      </defs>
      <rect x="16" y="16" width="480" height="480" rx="112" fill="url(#aura-logo-bg)" />
      <rect x="16" y="16" width="480" height="480" rx="112" fill="url(#aura-logo-glow)" />
      <circle cx="256" cy="256" r="118" fill="none" stroke="#ffffff" strokeWidth="36" strokeOpacity="0.95" />
      <circle cx="256" cy="256" r="46" fill="#ffffff" />
      <circle cx="256" cy="256" r="176" fill="none" stroke="#ffffff" strokeWidth="10" strokeOpacity="0.35" />
    </svg>
  );
}
