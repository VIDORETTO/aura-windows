// A minimal real MCP stdio server for native tests, written to a folder with
// spaces. It records the argv it received and offers tools `qa_echo` and
// `qa_write`; `fail` makes it exit with an error on start (status/logs tests).
import { mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";

const SERVER = String.raw`
import { appendFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
const dir = path.dirname(fileURLToPath(import.meta.url));
writeFileSync(path.join(dir, "argv.json"), JSON.stringify(process.argv.slice(2)));
if (process.argv.includes("--fail")) {
  console.error("QA MCP server failed on purpose: missing QA_TOKEN");
  process.exit(3);
}
let buf = "";
process.stdin.on("data", (d) => {
  buf += d;
  let i;
  while ((i = buf.indexOf("\n")) >= 0) {
    const line = buf.slice(0, i); buf = buf.slice(i + 1);
    if (!line.trim()) continue;
    const m = JSON.parse(line);
    appendFileSync(path.join(dir, "calls.log"), m.method + "\n");
    if (m.id === undefined) continue;
    const reply = (result) => process.stdout.write(JSON.stringify({ jsonrpc: "2.0", id: m.id, result }) + "\n");
    if (m.method === "initialize") reply({ protocolVersion: m.params.protocolVersion, capabilities: { tools: {} }, serverInfo: { name: "qa-mcp", version: "1.0.0" } });
    else if (m.method === "tools/list") reply({ tools: [
      { name: "qa_echo", description: "Echo for QA", inputSchema: { type: "object", properties: { text: { type: "string" } } } },
      { name: "qa_write", description: "Pretend write for QA", inputSchema: { type: "object", properties: {} } },
    ] });
    else if (m.method === "tools/call") reply({ content: [{ type: "text", text: "QA " + m.params.name }] });
    else process.stdout.write(JSON.stringify({ jsonrpc: "2.0", id: m.id, error: { code: -32601, message: "no" } }) + "\n");
  }
});
`;

export function writeMcpServer(root: string) {
  const dir = path.join(root, "qa mcp folder");
  mkdirSync(dir, { recursive: true });
  const file = path.join(dir, "qa-server.mjs");
  writeFileSync(file, SERVER);
  return { dir, file };
}
