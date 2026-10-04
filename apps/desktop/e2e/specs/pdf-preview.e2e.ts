// QA-028: a PDF in the Files panel shows a text preview (or an explicit
// explanation for a PDF without text) inside Aura.
import { readdirSync, statSync, writeFileSync } from "node:fs";
import path from "node:path";
import { responsesUpstream } from "../responses";

/** Minimal PDF: one Helvetica text line per page, xref offsets computed. */
function pdf(pages: string[]): Buffer {
  const n = pages.length;
  const objects = [
    "<< /Type /Catalog /Pages 2 0 R >>",
    `<< /Type /Pages /Kids [${pages.map((_, i) => `${4 + 2 * i} 0 R`).join(" ")}] /Count ${n} >>`,
    "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
  ];
  pages.forEach((text, i) => {
    objects.push(`<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 144] /Contents ${5 + 2 * i} 0 R /Resources << /Font << /F1 3 0 R >> >> >>`);
    const stream = text ? `BT /F1 12 Tf 20 100 Td (${text}) Tj ET` : "";
    objects.push(`<< /Length ${stream.length} >>\nstream\n${stream}\nendstream`);
  });
  let out = "%PDF-1.4\n";
  const offsets: number[] = [];
  objects.forEach((o, i) => { offsets.push(out.length); out += `${i + 1} 0 obj\n${o}\nendobj\n`; });
  const xref = out.length;
  out += `xref\n0 ${objects.length + 1}\n0000000000 65535 f \n` + offsets.map((o) => `${String(o).padStart(10, "0")} 00000 n \n`).join("");
  out += `trailer\n<< /Size ${objects.length + 1} /Root 1 0 R >>\nstartxref\n${xref}\n%%EOF\n`;
  return Buffer.from(out, "latin1");
}

describe("PDF preview (QA-028)", () => {
  let up: Awaited<ReturnType<typeof responsesUpstream>>;
  before(async () => { up = await responsesUpstream(() => "QA reply"); });
  after(async () => { await up.close(); });

  it("shows the text of the first pages and explains a scanned PDF", async () => {
    await browser.execute(async (url) => {
      const i = (window as any).__TAURI_INTERNALS__.invoke;
      await i("settings_update", { patch: { language: "en" } });
      for (const p of await i("providers_list")) await i("providers_remove", { id: p.id });
      await i("providers_save", { draft: { name: "QA pdf", preset: "ollama", baseUrl: url }, credential: null });
    }, up.baseUrl);
    await browser.refresh();
    const box = await $("textarea");
    await box.waitForDisplayed({ timeout: 20_000 });
    await box.setValue("QA pdf preview");
    await browser.keys("Enter");
    await browser.waitUntil(async () => (await browser.execute(() => document.body.innerText)).includes("QA reply"), { timeout: 150_000 });
    const root = path.join(process.env.AURA_HOME!, "workspaces");
    const workspace = readdirSync(root).map((d) => path.join(root, d)).filter((d) => statSync(d).isDirectory() && !d.endsWith("_ephemeral")).sort((a, b) => statSync(b).mtimeMs - statSync(a).mtimeMs)[0];
    writeFileSync(path.join(workspace, "qa-report.pdf"), pdf(Array.from({ length: 7 }, (_, i) => `QA PDF page ${i + 1}`)));
    writeFileSync(path.join(workspace, "qa-scan.pdf"), pdf(["", ""]));

    await (await $('button[aria-label="Files and changes"]')).click();
    await (await $("button*=qa-report.pdf")).click();
    const region = await $('section[aria-label="Preview of qa-report.pdf"]');
    await region.waitForDisplayed({ timeout: 10_000 });
    const text = await region.getText();
    console.log("Native PDF preview", JSON.stringify(text));
    expect(text).toContain("Text preview: 5 of 7 pages");
    expect(text).toContain("Page 1");
    expect(text).toContain("QA PDF page 5");
    expect(text).not.toContain("QA PDF page 6");
    await (await $("button*=qa-scan.pdf")).click();
    await $("p=This PDF has no selectable text (probably scanned). Use Open to see its pages.").waitForDisplayed({ timeout: 10_000 });
    await browser.saveScreenshot(path.resolve(process.cwd(), "../../../target/qa-tools/pdf-preview-native.png"));
  });
});
