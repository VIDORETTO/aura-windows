import { closeOpenFence, renderMarkdown } from "./markdown";

describe("markdown", () => {
  it("sanitizes scripts, event handlers and remote images", () => {
    const html = renderMarkdown('hi <script>alert(1)</script><img src="http://x/y.png" onerror="alert(1)"> ![a](http://x/a.png)');
    expect(html).not.toContain("<script");
    expect(html).not.toContain("onerror");
    expect(html).not.toContain("<img");
    expect(html).toContain("[imagem:");
  });

  it("turns links into data-href anchors (opened by the host)", () => {
    const html = renderMarkdown("[docs](https://example.com) [x](javascript:alert(1))");
    expect(html).toContain('data-href="https://example.com"');
    expect(html).not.toMatch(/\shref=/);
  });

  it("closes an unterminated fence while streaming", () => {
    expect(closeOpenFence("```rust\nfn main")).toBe("```rust\nfn main\n```");
    expect(closeOpenFence("ok")).toBe("ok");
    const html = renderMarkdown("```js\nlet a = 1", { streaming: true });
    expect(html).toContain("<pre");
  });

  it("renders GFM tables and lists", () => {
    const html = renderMarkdown("| a | b |\n|---|---|\n| 1 | 2 |\n\n- x\n- y");
    expect(html).toContain("<table>");
    expect(html).toContain("<li>x</li>");
  });
});
