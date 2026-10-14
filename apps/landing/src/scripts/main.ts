// The page's only script (PLANO.md §2): reveals, hero loop, sticky steps, glow, shortcuts,
// release refresh and copy buttons. Everything degrades to a complete static page without it.
import { API_LATEST, megabytes, newer, parseRelease, releaseDate, type Release } from "../lib/release";

const reduced = matchMedia("(prefers-reduced-motion: reduce)").matches;
const $$ = <T extends Element>(sel: string, root: ParentNode = document) => [...root.querySelectorAll<T>(sel)];

/* Nav background after the first pixels of scroll. */
const nav = document.querySelector("[data-nav]");
const onScroll = () => nav?.classList.toggle("scrolled", scrollY > 8);
addEventListener("scroll", onScroll, { passive: true });
onScroll();

/* reveal-up / stagger */
const reveal = new IntersectionObserver(
  (entries) => {
    for (const e of entries) {
      if (!e.isIntersecting) continue;
      e.target.classList.add("in");
      reveal.unobserve(e.target);
    }
  },
  { rootMargin: "0px 0px -8% 0px", threshold: 0.12 },
);
$$("[data-reveal]").forEach((el) => reveal.observe(el));

/* Hero loop runs only on screen; parallax follows a fine pointer. */
const demo = document.querySelector<HTMLElement>("[data-demo]");
if (demo && !reduced) {
  new IntersectionObserver(([e]) => demo.classList.toggle("playing", e.isIntersecting), { threshold: 0.2 }).observe(demo);
  if (matchMedia("(pointer: fine)").matches) {
    let frame = 0;
    addEventListener(
      "pointermove",
      (ev) => {
        cancelAnimationFrame(frame);
        frame = requestAnimationFrame(() => {
          demo.style.setProperty("--mx", (ev.clientX / innerWidth - 0.5).toFixed(3));
          demo.style.setProperty("--my", (ev.clientY / innerHeight - 0.5).toFixed(3));
        });
      },
      { passive: true },
    );
  }
}

/* sticky-swap: the step closest to the middle of the screen drives the visual. */
const stepsRoot = document.querySelector("[data-steps]");
if (stepsRoot) {
  const steps = $$<HTMLElement>("[data-step]", stepsRoot);
  const shots = $$<HTMLElement>(".shot", stepsRoot);
  const dots = $$<HTMLElement>(".dots span", stepsRoot);
  const set = (i: number) => {
    steps.forEach((s, j) => s.classList.toggle("active", j === i));
    shots.forEach((s, j) => s.classList.toggle("on", j === i));
    dots.forEach((s, j) => s.classList.toggle("on", j === i));
  };
  const io = new IntersectionObserver(
    (entries) => {
      for (const e of entries) if (e.isIntersecting) set(Number((e.target as HTMLElement).dataset.step));
    },
    { rootMargin: "-45% 0px -45% 0px" },
  );
  steps.forEach((s) => io.observe(s));
}

/* fan-out */
const fan = document.querySelector("[data-fan]");
if (fan) {
  const io = new IntersectionObserver(([e]) => {
    if (!e.isIntersecting) return;
    fan.classList.add("open");
    io.disconnect();
  }, { threshold: 0.35 });
  io.observe(fan);
}

/* glow-follow */
if (matchMedia("(pointer: fine)").matches) {
  $$<HTMLElement>("[data-glow]").forEach((card) => {
    card.addEventListener(
      "pointermove",
      (ev) => {
        const r = card.getBoundingClientRect();
        card.style.setProperty("--x", `${ev.clientX - r.left}px`);
        card.style.setProperty("--y", `${ev.clientY - r.top}px`);
      },
      { passive: true },
    );
  });
}

/* Shortcut keyboard */
const readout = document.querySelector<HTMLElement>("[data-readout]");
if (readout) {
  const [keys, action] = $$<HTMLElement>("span", readout);
  const combos = $$<HTMLButtonElement>("[data-combo]");
  const show = (b: HTMLButtonElement) => {
    if (b.classList.contains("on")) return;
    combos.forEach((c) => c.classList.toggle("on", c === b));
    keys.textContent = b.dataset.combo ?? "";
    action.textContent = b.dataset.action ?? "";
    readout.classList.remove("swap");
    void readout.offsetWidth;
    readout.classList.add("swap");
  };
  combos.forEach((b) => {
    b.addEventListener("pointerenter", () => show(b));
    b.addEventListener("focus", () => show(b));
    b.addEventListener("click", () => show(b));
  });
}

/* Copy buttons */
const copy = async (text: string, button: HTMLElement) => {
  try {
    await navigator.clipboard.writeText(text);
    button.classList.add("done");
    setTimeout(() => button.classList.remove("done"), 1600);
  } catch {
    /* clipboard denied: the text stays selectable on the page */
  }
};
document.querySelector<HTMLElement>("[data-copy-hash]")?.addEventListener("click", (e) => {
  const hash = document.querySelector("[data-hash]")?.textContent?.trim();
  if (hash) void copy(hash, e.currentTarget as HTMLElement);
});
document.querySelector<HTMLElement>("[data-copy-link]")?.addEventListener("click", (e) => {
  const button = e.currentTarget as HTMLElement;
  void copy(button.dataset.copyLink ?? location.href, button).then(() => {
    if (button.classList.contains("done")) button.textContent = "Link copiado";
  });
});

/* Not on Windows (phone, Mac, Linux): explain, without blocking the download. */
const ua = navigator.userAgent;
if (!/Windows/i.test(ua) || /Windows Phone/i.test(ua)) {
  $$<HTMLElement>("[data-platform]").forEach((p) => (p.hidden = false));
}

/* Release refresh: one API call per session; only a newer release than the build changes the page. */
const KEY = "aura-latest-release";
const apply = (r: Release) => {
  const built = document.querySelector<HTMLElement>('[data-dl="setup"][data-version]')?.dataset.version ?? "";
  if (!r.setup || (built && !newer(r.version, built))) return;
  $$<HTMLAnchorElement>('[data-dl="setup"]').forEach((a) => {
    a.href = r.setup!.url;
    a.dataset.version = r.version;
  });
  $$<HTMLAnchorElement>('[data-dl="msi"]').forEach((a) => r.msi && (a.href = r.msi.url));
  $$<HTMLElement>("[data-dl-meta]").forEach((m) => (m.textContent = `Windows 10/11 · x64 · v${r.version} · ${megabytes(r.setup!.bytes)} MB${m.dataset.suffix ?? ""}`));
  $$<HTMLElement>("[data-version-label]").forEach((m) => (m.textContent = ` · v${r.version}`));
  $$<HTMLAnchorElement>("[data-notes]").forEach((a) => (a.href = r.notesUrl));
  $$<HTMLElement>("[data-release-date]").forEach((m) => (m.textContent = `Publicada em ${releaseDate(r.publishedAt)}`));
  const row = document.querySelector<HTMLElement>("[data-hash-row]");
  const code = document.querySelector<HTMLElement>("[data-hash]");
  if (row && code) {
    code.textContent = r.setup.sha256 ?? "";
    row.hidden = !r.setup.sha256;
  }
};
(async () => {
  try {
    const cached = sessionStorage.getItem(KEY);
    if (cached) return apply(JSON.parse(cached) as Release);
  } catch {
    /* storage unavailable */
  }
  try {
    const res = await fetch(API_LATEST, { headers: { Accept: "application/vnd.github+json" } });
    if (!res.ok) return;
    const r = parseRelease(await res.json());
    if (!r) return;
    try {
      sessionStorage.setItem(KEY, JSON.stringify(r));
    } catch {
      /* storage unavailable */
    }
    apply(r);
  } catch {
    /* offline or rate limited: the build-time link stays */
  }
})();
