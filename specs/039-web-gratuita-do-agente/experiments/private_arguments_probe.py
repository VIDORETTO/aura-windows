"""Disposable protocol experiment: private arguments before Codex persistence.

Real pinned app-server; scripted loopback MCP/model; no search, inference key
or runtime Python dependency. Does not prove production authorization/budgets.
"""
import ast
import json
import os
from pathlib import Path
import queue
import secrets
import subprocess
import sys
import tempfile
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

from audit_synthetic_storage import audit

# Reuse only the stdio adapter class, preserving the already executed probe.
# No top-level code or legacy experiment is evaluated by this import.
legacy = ast.parse(Path(__file__).with_name("private_result_probe.py").read_text(encoding="utf-8"))
rpc_definition = next(node for node in legacy.body if isinstance(node, ast.ClassDef) and node.name == "Rpc")
exec(compile(ast.Module(body=[rpc_definition], type_ignores=[]), "probe_stdio_adapter", "exec"))

ROOT = Path(tempfile.mkdtemp(prefix="aura-039-private-arguments-", dir=os.environ["TEMP"]))
CODEX_HOME = ROOT / "codex-home"
WORK = ROOT / "workspace"
CODEX_HOME.mkdir()
WORK.mkdir()
URL = "https://independent.example/report"
REAL_ARGS = {"url": URL, "maxChars": 20000}
ARGS_HANDLE = secrets.token_hex(32)
RESULT_HANDLE = secrets.token_hex(32)
REASONING_HANDLE = secrets.token_hex(32)
flags = set(sys.argv[1:])
if flags - {"--reasoning", "--schema-conforming"}:
    raise SystemExit("usage: private_arguments_probe.py [--reasoning] [--schema-conforming]")
WITH_REASONING = "--reasoning" in flags
SCHEMA_CONFORMING = "--schema-conforming" in flags
ARGS = ({"url": "aura-tool-arguments:v1:" + ARGS_HANDLE} if SCHEMA_CONFORMING
        else {"auraToolArguments": {"version": 1, "handle": ARGS_HANDLE}})
RESULT = {"auraToolResult": {"version": 1, "handle": RESULT_HANDLE}}
REASONING = {"auraPrivateReasoning": {"version": 1, "handle": REASONING_HANDLE}}
PAGE = json.dumps({"text": "Production: 30 units.", "url": URL, "externalContent": True})
ANSWER = "Pesquisa concluida sem citar a pagina consultada."
STATE = {"requests": 0, "mcpCalls": 0, "receivedOpaqueArguments": False,
         "resolvedOriginalArguments": False, "restoredUpstreamArguments": False,
         "expandedUpstreamResult": False, "threadScopeMatches": False, "thread": None,
         "restoredUpstreamReasoning": False}


class Server(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, *_args):
        pass

    def reply(self, code, value, mime="application/json"):
        data = value.encode() if isinstance(value, str) else json.dumps(value).encode()
        self.send_response(code)
        self.send_header("Content-Type", mime)
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def do_DELETE(self):
        self.reply(204, "")

    def do_GET(self):
        self.reply(405, {})

    def do_POST(self):
        data = json.loads(self.rfile.read(int(self.headers.get("Content-Length", 0))))
        if self.path == "/mcp":
            method = data.get("method")
            if method == "notifications/initialized":
                self.reply(202, "")
                return
            if method == "initialize":
                result = {"protocolVersion": "2025-03-26", "capabilities": {"tools": {}},
                          "serverInfo": {"name": "probe", "version": "1"}}
            elif method == "tools/list":
                # Same required url/additionalProperties contract as Aura;
                # the experiment checks whether Codex forwards an opaque form.
                result = {"tools": [{"name": "web_fetch", "description": "Read a public page.",
                          "annotations": {"readOnlyHint": True, "destructiveHint": False, "openWorldHint": True},
                          "inputSchema": {"type": "object", "properties": {
                              "url": {"type": "string", "minLength": 1, "maxLength": 2048},
                              "maxChars": {"type": "integer", "minimum": 1, "maximum": 20000}},
                              "required": ["url"], "additionalProperties": False}}]}
            elif method == "tools/call":
                STATE["mcpCalls"] += 1
                args = data["params"].get("arguments")
                STATE["receivedOpaqueArguments"] = args == ARGS
                if args != ARGS:
                    result = {"isError": True, "content": [{"type": "text", "text": "Invalid synthetic reference"}]}
                else:
                    # Trusted emitter's memory supplies the real tool input.
                    STATE["resolvedOriginalArguments"] = REAL_ARGS["url"] == URL
                    result = {"isError": False, "content": [{"type": "text", "text": json.dumps(RESULT)}]}
            else:
                result = {}
            self.reply(200, {"jsonrpc": "2.0", "id": data.get("id"), "result": result})
            return
        if self.path != "/v1/responses":
            self.reply(404, {})
            return
        STATE["requests"] += 1
        STATE["threadScopeMatches"] = self.headers.get("thread-id") == STATE["thread"]
        outputs = [item for item in data.get("input", []) if item.get("type") == "function_call_output"]
        if outputs:
            for item in data.get("input", []):
                if WITH_REASONING and item.get("type") == "reasoning" and item.get("id") == "rs_probe":
                    # Opaque field is a synthetic stand-in, not a real provider
                    # cryptographic blob. Test preservation/forwarding only.
                    if item.get("encrypted_content") == json.dumps(REASONING):
                        item["encrypted_content"] = "synthetic-original-opaque-reasoning"
                        item["summary"] = [{"type": "summary_text", "text": PAGE}]
                        STATE["restoredUpstreamReasoning"] = True
                if item.get("type") == "function_call" and item.get("call_id") == "call_probe":
                    if json.loads(item["arguments"]) == ARGS:
                        item["arguments"] = json.dumps(REAL_ARGS)
                        STATE["restoredUpstreamArguments"] = json.loads(item["arguments"]) == REAL_ARGS
            value = outputs[-1].get("output")
            blocks = [{"text": value}] if isinstance(value, str) else value
            for block in blocks:
                try:
                    reference = json.loads(block.get("text", ""))
                except (ValueError, TypeError):
                    continue
                if reference == RESULT:
                    block["text"] = PAGE
                    STATE["expandedUpstreamResult"] = "Production: 30 units." in block["text"]
            if not STATE["restoredUpstreamArguments"] or not STATE["expandedUpstreamResult"]:
                self.reply(400, {"error": {"code": "probe_failed", "message": "Synthetic rehydration failed"}})
                return
            item = {"id": "msg_probe", "type": "message", "status": "completed", "role": "assistant",
                    "content": [{"type": "output_text", "text": ANSWER, "annotations": []}]}
        else:
            name, namespace = None, None
            for tool in data.get("tools", []):
                if tool.get("type") == "namespace":
                    for inner in tool.get("tools", []):
                        if inner.get("name") == "web_fetch":
                            name, namespace = inner["name"], tool["name"]
                elif tool.get("name") == "mcp__probe__web_fetch":
                    name = tool["name"]
            assert name, "MCP definition missing"
            # Model-generated arguments are retained only in this fixture's
            # memory; every response event sent to Codex has the opaque form.
            item = {"id": "fc_probe", "type": "function_call", "status": "completed",
                    "call_id": "call_probe", "name": name, "arguments": json.dumps(ARGS)}
            if namespace:
                item["namespace"] = namespace
        items = [item]
        if WITH_REASONING and not outputs:
            items.insert(0, {"type": "reasoning", "id": "rs_probe", "summary": [
                {"type": "summary_text", "text": json.dumps(REASONING)}],
                "encrypted_content": json.dumps(REASONING)})
        events = [("response.created", {"type": "response.created", "response": {
            "id": "resp_probe", "object": "response", "status": "in_progress", "output": []}})]
        for index, event_item in enumerate(items):
            events.extend([
                ("response.output_item.added", {"type": "response.output_item.added", "output_index": index, "item": event_item}),
                ("response.output_item.done", {"type": "response.output_item.done", "output_index": index, "item": event_item}),
            ])
        events.append(("response.completed", {"type": "response.completed", "response": {
            "id": "resp_probe", "object": "response", "status": "completed", "output": items,
            "usage": {"input_tokens": 1, "output_tokens": 1, "total_tokens": 2}}}))
        self.reply(200, "".join("event: " + name + "\ndata: " + json.dumps(value) + "\n\n" for name, value in events), "text/event-stream")


server = ThreadingHTTPServer(("127.0.0.1", 0), Server)
threading.Thread(target=server.serve_forever, daemon=True).start()
port = server.server_address[1]
(CODEX_HOME / "config.toml").write_text(f'''model_provider = "probe"
model = "gpt-5.2"
web_search = "disabled"
[model_providers.probe]
name = "Probe"
base_url = "http://127.0.0.1:{port}/v1"
wire_api = "responses"
requires_openai_auth = false
supports_websockets = false
[features]
memories = false
[mcp_servers.probe]
url = "http://127.0.0.1:{port}/mcp"
default_tools_approval_mode = "auto"
''', encoding="utf-8")

result = {"kind": "prototype-only", "paidInference": False, "root": str(ROOT),
          "argumentsConformToAdvertisedSchema": SCHEMA_CONFORMING,
          "reasoningExercised": WITH_REASONING, "realProviderEncryptedBlobExercised": False,
          "productionBudgetAuthorizationExercised": False}
rpc = Rpc()
try:
    started = rpc.call("thread/start", {"model": "gpt-5.2", "modelProvider": "probe", "cwd": str(WORK),
                       "approvalPolicy": "never", "sandbox": "read-only", "ephemeral": False})
    assert "result" in started, "thread/start failed"
    thread_id = started["result"]["thread"]["id"]
    STATE["thread"] = thread_id
    turn = rpc.call("turn/start", {"threadId": thread_id, "input": [{"type": "text", "text": "Pesquise um exemplo publico sem citar a pagina."}]})
    assert "result" in turn, "turn/start failed"
    result["turnStatus"] = rpc.finish_turn()
finally:
    rpc.close()
rpc = Rpc()
try:
    restored = rpc.call("thread/read", {"threadId": thread_id, "includeTurns": True})
    result["sameThreadAfterRestart"] = restored.get("result", {}).get("thread", {}).get("id") == thread_id
    result["answerRestored"] = ANSWER in json.dumps(restored.get("result", {}))
finally:
    rpc.close()
    server.shutdown()
storage = audit(ROOT)
if WITH_REASONING:
    storage["limitations"] = [value for value in storage["limitations"]
        if value != "Reasoning outputs and compaction are not exercised by this fixture"]
    storage["limitations"].append("Synthetic reasoning/opaque field only; no real cryptographic blob or compaction")
result["storage"] = storage
result["opaqueArgumentsOnDisk"] = any(ARGS_HANDLE.encode() in path.read_bytes() for path in CODEX_HOME.rglob("*") if path.is_file())
result.update({key: value for key, value in STATE.items() if key != "thread"})
(ROOT / "result.json").write_text(json.dumps(result, indent=2), encoding="utf-8")
print(json.dumps(result, indent=2))
assert result["turnStatus"] == "completed" and STATE["mcpCalls"] == 1 and STATE["requests"] == 2
assert STATE["receivedOpaqueArguments"] and STATE["resolvedOriginalArguments"]
assert STATE["restoredUpstreamArguments"] and STATE["expandedUpstreamResult"] and STATE["threadScopeMatches"]
assert not WITH_REASONING or STATE["restoredUpstreamReasoning"], "Opaque reasoning was not retained for the upstream"
assert result["sameThreadAfterRestart"] and result["answerRestored"] and result["opaqueArgumentsOnDisk"]
assert not storage["hits"], "Synthetic URL or page content reached persistent storage"
