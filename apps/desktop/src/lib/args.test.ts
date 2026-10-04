import { formatArgs, parseArgs } from "./args";

describe("command-line arguments for MCP servers", () => {
  it("keeps quoted Windows paths with spaces as one argument", () => {
    expect(parseArgs('"C:\\QA Folder\\server.js" --port 3000')).toEqual({ ok: true, args: ["C:\\QA Folder\\server.js", "--port", "3000"] });
  });

  it("keeps empty quoted arguments", () => {
    expect(parseArgs('--name "" --other \'\'')).toEqual({ ok: true, args: ["--name", "", "--other", ""] });
  });

  it("supports both quote styles and escaped double quotes", () => {
    expect(parseArgs("--msg 'diz \"oi\"'")).toEqual({ ok: true, args: ["--msg", 'diz "oi"'] });
    expect(parseArgs('--q "a \\"b\\" c"')).toEqual({ ok: true, args: ["--q", 'a "b" c'] });
  });

  it("joins adjacent quoted and plain parts and ignores extra whitespace", () => {
    expect(parseArgs('  --path="C:\\Program Files\\x"   -y ')).toEqual({ ok: true, args: ["--path=C:\\Program Files\\x", "-y"] });
    expect(parseArgs("")).toEqual({ ok: true, args: [] });
  });

  it("reports an unterminated quote", () => {
    expect(parseArgs('"C:\\QA Folder\\server.js --port')).toEqual({ ok: false, error: "unterminated" });
  });

  it("formats arguments back so they parse to the same list", () => {
    const args = ["C:\\QA Folder\\server.js", "", 'diz "oi"', "-y"];
    expect(formatArgs(args)).toBe('"C:\\QA Folder\\server.js" "" "diz \\"oi\\"" -y');
    expect(parseArgs(formatArgs(args))).toEqual({ ok: true, args });
  });
});
