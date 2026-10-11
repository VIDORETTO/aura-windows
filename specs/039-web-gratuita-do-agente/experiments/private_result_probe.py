import json, os, pathlib, queue, subprocess, tempfile, threading, time, uuid
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

ROOT = pathlib.Path(tempfile.mkdtemp(prefix='aura-039-private-result-', dir=os.environ['TEMP']))
CODEX_HOME = ROOT / 'codex-home'
CODEX_HOME.mkdir()
WORK = ROOT / 'workspace'
WORK.mkdir()
CAP = uuid.uuid4().hex
PAGE = 'UNCITED_PAGE_PRIVATE_SENTINEL_039: Production was 30 units and this entire paragraph must not be recorded.'
ANSWER = 'Pesquisa concluida.'
STATE = {'requests': 0, 'mcp_calls': 0, 'expanded': 0, 'headers_matching_thread': [], 'output_type': None, 'thread': None}

class Server(BaseHTTPRequestHandler):
    protocol_version = 'HTTP/1.1'
    def log_message(self, *args):
        pass
    def reply(self, code, payload, mime='application/json'):
        data = payload.encode() if isinstance(payload, str) else json.dumps(payload).encode()
        self.send_response(code)
        self.send_header('Content-Type', mime)
        self.send_header('Content-Length', str(len(data)))
        self.end_headers()
        self.wfile.write(data)
    def do_GET(self):
        self.reply(405, {})
    def do_DELETE(self):
        self.reply(204, '')
    def do_POST(self):
        data = json.loads(self.rfile.read(int(self.headers.get('Content-Length', 0))))
        if self.path == '/mcp':
            method = data.get('method')
            if method == 'notifications/initialized':
                self.reply(202, '')
                return
            if method == 'initialize':
                result = {'protocolVersion': '2025-03-26', 'capabilities': {'tools': {}}, 'serverInfo': {'name': 'probe', 'version': '1'}}
            elif method == 'tools/list':
                result = {'tools': [{'name': 'web_fetch', 'description': 'Read a public page.', 'annotations': {'readOnlyHint': True, 'openWorldHint': False}, 'inputSchema': {'type':'object', 'properties': {}, 'additionalProperties': False}}]}
            elif method == 'tools/call':
                STATE['mcp_calls'] += 1
                result = {'content': [{'type':'text', 'text': json.dumps({'aura_private_tool_result': CAP})}], 'isError': False}
            else:
                result = {}
            self.reply(200, {'jsonrpc':'2.0', 'id':data.get('id'), 'result':result})
            return
        if self.path != '/v1/responses':
            self.reply(404, {})
            return
        STATE['requests'] += 1
        STATE['headers_matching_thread'] = [k.lower() for k, v in self.headers.items() if v == STATE['thread']]
        outputs = [x for x in data.get('input', []) if x.get('type') == 'function_call_output']
        if outputs:
            last = outputs[-1]
            STATE['output_type'] = type(last.get('output')).__name__
            # Experiment only: a strict handle inside the exact tool output, never a user message.
            payload = last.get('output')
            def expand(value, depth=0):
                if depth > 8:
                    return value
                if value == {'aura_private_tool_result': CAP}:
                    STATE['expanded'] += 1
                    return PAGE
                if isinstance(value, str):
                    try:
                        decoded = json.loads(value)
                    except (ValueError, TypeError):
                        return value
                    if isinstance(decoded, (dict,list)):
                        return json.dumps(expand(decoded, depth+1))
                    return value
                if isinstance(value, list):
                    return [expand(x, depth+1) for x in value]
                if isinstance(value, dict):
                    return {k:expand(v, depth+1) for k,v in value.items()}
                return value
            payload = expand(payload)
            if PAGE not in json.dumps(payload):
                STATE['expansion_failed'] = True
                STATE['output_has_handle'] = CAP in json.dumps(last.get('output'))
                self.reply(400, {'error':{'code':'probe_expansion_failed','message':'Synthetic prototype failed to expand the MCP result.'}})
                return
            item = {'id':'msg_probe', 'type':'message', 'status':'completed', 'role':'assistant', 'content':[{'type':'output_text', 'text':ANSWER, 'annotations':[]}]}
        else:
            name, namespace = None, None
            for tool in data.get('tools', []):
                if tool.get('type') == 'namespace':
                    for inner in tool.get('tools', []):
                        if inner.get('name') == 'web_fetch':
                            name, namespace = inner['name'], tool['name']
                elif tool.get('name') == 'mcp__probe__web_fetch':
                    name = tool['name']
            assert name, 'MCP tool definition not supplied'
            item = {'id':'fc_probe', 'type':'function_call', 'status':'completed', 'call_id':'call_probe', 'name':name, 'arguments':'{}'}
            if namespace:
                item['namespace'] = namespace
        events = [
            ('response.created', {'type':'response.created', 'response':{'id':'resp_probe','object':'response','status':'in_progress','output':[]}}),
            ('response.output_item.added', {'type':'response.output_item.added','output_index':0,'item':item}),
            ('response.output_item.done', {'type':'response.output_item.done','output_index':0,'item':item}),
            ('response.completed', {'type':'response.completed','response':{'id':'resp_probe','object':'response','status':'completed','output':[item],'usage':{'input_tokens':1,'output_tokens':1,'total_tokens':2}}})
        ]
        self.reply(200, ''.join('event: '+k+'\ndata: '+json.dumps(v)+'\n\n' for k,v in events), 'text/event-stream')

server = ThreadingHTTPServer(('127.0.0.1', 0), Server)
threading.Thread(target=server.serve_forever, daemon=True).start()
port = server.server_address[1]
(CODEX_HOME / 'config.toml').write_text(f'''model_provider = "probe"
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
''', encoding='utf-8')

class Rpc:
    def __init__(self):
        env = os.environ.copy()
        env['CODEX_HOME'] = str(CODEX_HOME)
        env.pop('AURA_GATEWAY_DUMP_DIR', None)
        self.proc = subprocess.Popen([os.environ['AURA_CODEX_BIN'], '--listen', 'stdio://'], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True, encoding='utf-8', env=env, cwd=WORK, creationflags=subprocess.CREATE_NO_WINDOW)
        self.q = queue.Queue()
        self.ident = 0
        self.events = []
        threading.Thread(target=self.read, daemon=True).start()
        self.call('initialize', {'clientInfo':{'name':'aura_persistence_probe','title':'Aura persistence probe','version':'1'}, 'capabilities':{'experimentalApi':True}})
        self.send({'method':'initialized','params':{}})
    def read(self):
        for line in self.proc.stdout:
            try:
                self.q.put(json.loads(line))
            except ValueError:
                pass
    def send(self, value):
        self.proc.stdin.write(json.dumps(value)+'\n')
        self.proc.stdin.flush()
    def call(self, method, params):
        self.ident += 1
        ident = self.ident
        self.send({'id':ident,'method':method,'params':params})
        end = time.monotonic()+30
        while time.monotonic()<end:
            msg = self.q.get(timeout=max(0.01,end-time.monotonic()))
            if msg.get('id') == ident:
                return msg
            self.events.append(msg)
        raise TimeoutError(method)
    def finish_turn(self):
        end = time.monotonic()+30
        while time.monotonic()<end:
            msg = self.q.get(timeout=max(0.01,end-time.monotonic()))
            if msg.get('method') == 'turn/completed':
                return msg['params']['turn']['status']
            self.events.append(msg)
        raise TimeoutError('turn/completed')
    def close(self):
        self.proc.stdin.close()
        try:
            self.proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            self.proc.kill()
            self.proc.wait(timeout=5)

result = {'root': str(ROOT), 'kind':'prototype-only', 'paid_inference':False}
rpc = Rpc()
try:
    start = rpc.call('thread/start', {'model':'gpt-5.2','modelProvider':'probe','cwd':str(WORK),'approvalPolicy':'never','sandbox':'read-only','ephemeral':False})
    assert 'result' in start, 'thread/start failed: '+str(start.get('error',{}).get('code'))
    tid = start['result']['thread']['id']
    STATE['thread'] = tid
    turn = rpc.call('turn/start', {'threadId':tid, 'input':[{'type':'text','text':'Pesquisar um exemplo publico.'}]})
    assert 'result' in turn, 'turn/start failed'
    result['turn_status'] = rpc.finish_turn()
finally:
    rpc.close()
files = [p for p in CODEX_HOME.rglob('*') if p.is_file()]
result['private_page_on_disk'] = any(PAGE.encode() in p.read_bytes() for p in files)
result['handle_on_disk'] = any(CAP.encode() in p.read_bytes() for p in files)
result['rollout_files'] = len(list(CODEX_HOME.rglob('*.jsonl')))
rpc = Rpc()
try:
    opened = rpc.call('thread/read', {'threadId':tid, 'includeTurns':True})
    result['same_thread_after_restart'] = opened.get('result',{}).get('thread',{}).get('id') == tid
    result['answer_in_restored_history'] = ANSWER in json.dumps(opened.get('result',{}))
finally:
    rpc.close()
server.shutdown()
result.update({k:v for k,v in STATE.items() if k != 'thread'})
(ROOT / 'result.json').write_text(json.dumps(result, indent=2), encoding='utf-8')
print(json.dumps(result, indent=2))
assert result['turn_status'] == 'completed'
assert STATE['requests'] == 2 and STATE['mcp_calls'] == 1 and STATE['expanded'] == 1
assert not result['private_page_on_disk'] and result['handle_on_disk']
assert result['same_thread_after_restart'] and result['answer_in_restored_history']


