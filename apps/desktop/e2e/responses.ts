// Loopback Responses-API upstream for native tests: deterministic replies,
// no inference, no account. Records every request body.
import { createServer, type IncomingMessage } from "node:http";

export interface Recorded {
  url: string;
  body: any;
  headers: Record<string, string | string[] | undefined>;
}

const read = (req: IncomingMessage) =>
  new Promise<string>((resolve) => {
    let data = "";
    req.on("data", (c) => (data += c));
    req.on("end", () => resolve(data));
  });

/** A tool call the loopback model makes instead of answering. */
export interface FunctionCall {
  functionCall: { name: string; arguments: string; namespace?: string };
}

/** `reply(body)` returns the assistant text (or a tool call) for one request. */
/** `models: null` = a provider without `/models` (404). */
export async function responsesUpstream(reply: (body: any) => string | FunctionCall = () => "QA reply", models: unknown[] | null = [{ id: "qa-model" }]) {
  const requests: Recorded[] = [];
  let seq = 0;
  const server = createServer(async (req, res) => {
    if (req.url === "/v1/models") {
      req.resume();
      requests.push({ url: req.url, body: null, headers: req.headers });
      if (models === null) {
        res.writeHead(404);
        res.end();
        return;
      }
      res.writeHead(200, { "content-type": "application/json" });
      res.end(JSON.stringify({ data: models }));
      return;
    }
    const raw = await read(req);
    let body: any = null;
    try { body = JSON.parse(raw); } catch { /* not JSON */ }
    requests.push({ url: req.url ?? "", body, headers: req.headers });
    if (req.url !== "/v1/responses") {
      res.writeHead(404);
      res.end();
      return;
    }
    const id = `resp_qa_${++seq}`;
    const out = reply(body);
    const send = (type: string, data: Record<string, unknown>) => res.write(`event: ${type}\ndata: ${JSON.stringify({ type, ...data })}\n\n`);
    res.writeHead(200, { "content-type": "text/event-stream", "cache-control": "no-cache" });
    send("response.created", { response: { id } });
    if (typeof out !== "string") {
      const item = { type: "function_call", id: `fc_${id}`, call_id: `call_${id}`, name: out.functionCall.name, arguments: out.functionCall.arguments, ...(out.functionCall.namespace ? { namespace: out.functionCall.namespace } : {}) };
      send("response.output_item.added", { output_index: 0, item: { ...item, arguments: "" } });
      send("response.output_item.done", { output_index: 0, item });
      send("response.completed", { response: { id, usage: { input_tokens: 100, input_tokens_details: { cached_tokens: 0 }, output_tokens: 5, output_tokens_details: { reasoning_tokens: 0 }, total_tokens: 105 } } });
      res.end();
      return;
    }
    const text = out;
    send("response.output_item.added", { output_index: 0, item: { type: "message", role: "assistant", id: `msg_${id}`, content: [] } });
    send("response.output_text.delta", { output_index: 0, content_index: 0, item_id: `msg_${id}`, delta: text });
    send("response.output_item.done", { output_index: 0, item: { type: "message", role: "assistant", id: `msg_${id}`, content: [{ type: "output_text", text }] } });
    send("response.completed", { response: { id, usage: { input_tokens: 100, input_tokens_details: { cached_tokens: 0 }, output_tokens: 5, output_tokens_details: { reasoning_tokens: 0 }, total_tokens: 105 } } });
    res.end();
  });
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();
  if (!address || typeof address === "string") throw new Error("QA upstream has no loopback port");
  return {
    baseUrl: `http://127.0.0.1:${address.port}/v1`,
    requests,
    close: () => new Promise<void>((resolve) => server.close(() => resolve())),
  };
}
