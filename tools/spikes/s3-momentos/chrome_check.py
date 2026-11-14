"""Spike S3 (esforço 046) — PROTÓTIPO. Verificações AC-011 no Chrome, com perfil
temporário e página leve (example.com):
  (a) ler só o nome UIA da raiz liga "Web accessibility"?
  (b) depois de uma leitura UIA completa, quanto tempo até o Chrome desligar o modo?
Uso: python -I chrome_check.py <s3-momentos.exe> <saida.json>
"""
import json, subprocess, sys, tempfile, time, urllib.request
import websocket

CHROME = r"C:\Program Files\Google\Chrome\Application\chrome.exe"
probe, out = sys.argv[1], sys.argv[2]
port = 9370
prof = tempfile.mkdtemp(prefix="s3chrome-")
p = subprocess.Popen([CHROME, f"--user-data-dir={prof}", f"--remote-debugging-port={port}", "--no-first-run",
                      "--no-default-browser-check", "--window-size=1000,700", "https://example.com/"])
res = {"chrome": None, "etapas": []}


def flags():
    tabs = json.load(urllib.request.urlopen(f"http://127.0.0.1:{port}/json", timeout=10))
    acc = next((t for t in tabs if t["url"].startswith("chrome://accessibility")), None)
    if acc is None:
        acc = json.load(urllib.request.urlopen(urllib.request.Request(f"http://127.0.0.1:{port}/json/new?chrome://accessibility", method="PUT"), timeout=10))
        time.sleep(2)
    ws = websocket.create_connection(acc["webSocketDebuggerUrl"], timeout=20)
    ws.send(json.dumps({"id": 1, "method": "Page.reload", "params": {}}))
    while json.loads(ws.recv()).get("id") != 1:
        pass
    time.sleep(2)
    ws.send(json.dumps({"id": 2, "method": "Runtime.evaluate", "params": {"returnByValue": True, "expression": """(() => { const o = {};
      document.querySelectorAll('input[type=checkbox]').forEach(x => { const l = ((x.closest('label')||x.parentElement||{}).innerText||x.id||'').trim().slice(0,40); o[l] = x.checked; });
      return o; })()"""}}))
    while True:
        m = json.loads(ws.recv())
        if m.get("id") == 2:
            ws.close()
            return m["result"]["result"].get("value") or {}


def activate_example():
    tabs = json.load(urllib.request.urlopen(f"http://127.0.0.1:{port}/json", timeout=10))
    ex = next(t for t in tabs if t["type"] == "page" and "example.com" in t["url"])
    urllib.request.urlopen(f"http://127.0.0.1:{port}/json/activate/{ex['id']}", timeout=10).read()
    time.sleep(1)


def probe_run(cmd):
    activate_example()
    r = subprocess.run([probe, cmd, "Example Domain"], capture_output=True, text=True)
    return (r.stdout.strip() or r.stderr.strip())[-200:]


def web_on(f):
    return any(v for k, v in f.items() if k.lower().startswith("web accessibility"))


def step(name, extra=None):
    f = flags()
    res["etapas"].append({"etapa": name, "t": round(time.time() - t0, 1), "web_accessibility": web_on(f),
                          "ligados": [k for k, v in f.items() if v], **(extra or {})})
    print(res["etapas"][-1], flush=True)


try:
    time.sleep(6)
    ver = json.load(urllib.request.urlopen(f"http://127.0.0.1:{port}/json/version", timeout=10))
    res["chrome"] = ver.get("Browser")
    t0 = time.time()
    step("antes")
    # (a) só o nome da raiz
    r = probe_run("uia-nome")
    time.sleep(3)
    step("depois_de_ler_so_o_nome", {"probe": r})
    # (b) leitura completa e espera pelo desligamento
    r = probe_run("uia-ler")
    time.sleep(3)
    step("depois_da_leitura_completa", {"probe": r})
    t_full = time.time()
    while time.time() - t_full < 600:
        time.sleep(10)
        step("espera")
        if not res["etapas"][-1]["web_accessibility"]:
            res["desligou_apos_s"] = round(time.time() - t_full)
            break
    else:
        res["desligou_apos_s"] = None
finally:
    try:
        ws = websocket.create_connection(json.load(urllib.request.urlopen(f"http://127.0.0.1:{port}/json/version", timeout=10))["webSocketDebuggerUrl"], timeout=10)
        ws.send(json.dumps({"id": 9, "method": "Browser.close"}))
    except Exception:
        p.kill()
    json.dump(res, open(out, "w", encoding="utf-8"), ensure_ascii=False, indent=1)
