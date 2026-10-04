import { createServer } from "node:http";

/** Loopback-only upstream: the real host/gateway/app-server still run normally. */
export async function localProvider() {
  let unavailable = false;
  const server = createServer((req, res) => {
    if (req.url === "/v1/models") {
      if (unavailable) {
        res.writeHead(503, { "content-type": "application/json" });
        res.end(JSON.stringify({ error: "QA controlled unavailable" }));
        return;
      }
      res.writeHead(200, { "content-type": "application/json" });
      res.end(JSON.stringify({ data: [{ id: "qa-literal-model", display_name: "QA literal model" }] }));
    } else if (req.url === "/v1/chat/completions") {
      req.resume();
      res.writeHead(200, { "content-type": "text/event-stream" });
      const chunk = (delta: unknown, finish_reason: string | null) =>
        `data: ${JSON.stringify({ id: "qa-response", object: "chat.completion.chunk", model: "qa-literal-model", choices: [{ index: 0, delta, finish_reason }] })}\n\n`;
      res.end(chunk({ role: "assistant", content: "QA controlled response" }, null) + chunk({}, "stop") + "data: [DONE]\n\n");
    } else {
      res.writeHead(404);
      res.end();
    }
  });
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();
  if (!address || typeof address === "string") throw new Error("QA provider has no loopback port");
  return {
    baseUrl: `http://127.0.0.1:${address.port}/v1`,
    fail: () => { unavailable = true; },
    close: () => new Promise<void>((resolve, reject) => server.close((error) => error ? reject(error) : resolve())),
  };
}
