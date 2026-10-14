// Static landing page published on GitHub Pages (see PLANO.md §9).
import { defineConfig } from "astro/config";
import sitemap from "@astrojs/sitemap";

export default defineConfig({
  site: process.env.LANDING_SITE ?? "https://vidoretto.github.io",
  base: process.env.LANDING_BASE ?? "/aura-windows",
  trailingSlash: "ignore",
  integrations: [sitemap()],
  build: { inlineStylesheets: "always" },
  prefetch: false,
  devToolbar: { enabled: false },
});
