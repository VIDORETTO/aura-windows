//! Spike S3/S4 (esforço 046) — PROTÓTIPO, fora do produto.
//! Ferramenta de campo do ADR 0013. Grava só números em
//! `%LOCALAPPDATA%\aura-spikes\s3\resumo-<início>.json`; nunca texto nem imagem.
//!
//! s3-momentos rodar [horas]      (padrão 10 h; para com Ctrl+C ou criando o arquivo `parar`)
//! s3-momentos uia-nome "<título>" lê só o nome UIA da raiz (verificação do Chrome)
//! s3-momentos uia-ler "<título>"  leitura completa (algoritmo do aura-win)
use s3_momentos::*;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use windows::Foundation::TypedEventHandler;
use windows::Globalization::Language;
use windows::Graphics::Capture::{Direct3D11CaptureFramePool, GraphicsCaptureItem, GraphicsCaptureSession};
use windows::Graphics::DirectX::Direct3D11::IDirect3DDevice;
use windows::Graphics::DirectX::DirectXPixelFormat;
use windows::Graphics::Imaging::{BitmapAlphaMode, BitmapPixelFormat, SoftwareBitmap};
use windows::Media::Ocr::OcrEngine;
use windows::Security::Cryptography::CryptographicBuffer;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_HARDWARE;
use windows::Win32::Graphics::Direct3D11::{D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION, D3D11CreateDevice, ID3D11Device};
use windows::Win32::Graphics::Dxgi::IDXGIDevice;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::Storage::Xps::{PRINT_WINDOW_FLAGS, PrintWindow};
use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx};
use windows::Win32::System::Console::SetConsoleCtrlHandler;
use windows::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
use windows::Win32::System::Threading::*;
use windows::Win32::System::WinRT::Direct3D11::CreateDirect3D11DeviceFromDXGIDevice;
use windows::Win32::System::WinRT::Graphics::Capture::IGraphicsCaptureItemInterop;
use windows::Win32::UI::Accessibility::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::{HSTRING, IInspectable, Interface, PWSTR};

static STOP: AtomicBool = AtomicBool::new(false);
static EV_FG: AtomicU64 = AtomicU64::new(0);
static EV_NAME: AtomicU64 = AtomicU64::new(0);
static EV_TITLE: AtomicU64 = AtomicU64::new(0);
static TX: OnceLock<Mutex<Sender<isize>>> = OnceLock::new();

const SETTLE: Duration = Duration::from_millis(1500);
const UIA_BUDGET: Duration = Duration::from_millis(200);
const HEARTBEAT: Duration = Duration::from_secs(60);
const SAVE_EVERY: Duration = Duration::from_secs(300);
const MAX_NODES: i32 = 2000;
const MAX_CHARS: usize = 20_000;

unsafe extern "system" fn hook(_h: HWINEVENTHOOK, ev: u32, hwnd: HWND, idobj: i32, idchild: i32, _t: u32, _ms: u32) {
    match ev {
        EVENT_SYSTEM_FOREGROUND => {
            EV_FG.fetch_add(1, Ordering::Relaxed);
            if let Some(tx) = TX.get() {
                let _ = tx.lock().unwrap().send(hwnd.0 as isize);
            }
        }
        EVENT_OBJECT_NAMECHANGE => {
            EV_NAME.fetch_add(1, Ordering::Relaxed);
            if idobj == OBJID_WINDOW.0 && idchild == CHILDID_SELF as i32 && unsafe { GetForegroundWindow() } == hwnd {
                EV_TITLE.fetch_add(1, Ordering::Relaxed);
                if let Some(tx) = TX.get() {
                    let _ = tx.lock().unwrap().send(hwnd.0 as isize);
                }
            }
        }
        _ => {}
    }
}

unsafe extern "system" fn ctrl(_t: u32) -> windows_core::BOOL {
    STOP.store(true, Ordering::SeqCst);
    true.into()
}

fn exe_of(hwnd: HWND) -> String {
    unsafe {
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        let Ok(h) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else { return "?".into() };
        let mut buf = [0u16; 512];
        let mut n = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut n).is_ok();
        let _ = CloseHandle(h);
        if !ok {
            return "?".into();
        }
        String::from_utf16_lossy(&buf[..n as usize]).rsplit('\\').next().unwrap_or("?").to_lowercase()
    }
}

fn class_of(hwnd: HWND) -> String {
    let mut b = [0u16; 256];
    let n = unsafe { GetClassNameW(hwnd, &mut b) };
    String::from_utf16_lossy(&b[..n.max(0) as usize])
}

/// Mesmo algoritmo de `aura-win::uia::window_text`; devolve só o número de caracteres.
fn uia_chars(uia: &IUIAutomation, hwnd: HWND) -> usize {
    unsafe {
        let Ok(root) = uia.ElementFromHandle(hwnd) else { return 0 };
        if let Ok(p) = root.GetCurrentPattern(UIA_TextPatternId)
            && let Ok(tp) = p.cast::<IUIAutomationTextPattern>()
            && let Ok(r) = tp.DocumentRange()
            && let Ok(t) = r.GetText(MAX_CHARS as i32)
            && !t.is_empty()
        {
            return t.to_string().chars().count();
        }
        let Ok(cond) = uia.CreateTrueCondition() else { return 0 };
        let Ok(all) = root.FindAll(TreeScope_Descendants, &cond) else { return 0 };
        let n = all.Length().unwrap_or(0).min(MAX_NODES);
        let mut total = 0usize;
        for i in 0..n {
            if total >= MAX_CHARS {
                break;
            }
            let Ok(el) = all.GetElement(i) else { continue };
            let Ok(kind) = el.CurrentControlType() else { continue };
            if (kind == UIA_DocumentControlTypeId || kind == UIA_EditControlTypeId)
                && let Ok(p) = el.GetCurrentPattern(UIA_TextPatternId)
                && let Ok(tp) = p.cast::<IUIAutomationTextPattern>()
                && let Ok(r) = tp.DocumentRange()
                && let Ok(t) = r.GetText(MAX_CHARS as i32)
                && !t.is_empty()
            {
                total += t.to_string().chars().count();
                continue;
            }
            if (kind == UIA_TextControlTypeId || kind == UIA_EditControlTypeId || kind == UIA_DocumentControlTypeId)
                && let Ok(name) = el.CurrentName()
            {
                total += name.to_string().chars().count();
            }
        }
        total.min(MAX_CHARS)
    }
}

/// Captura a janela inteira na resolução nativa (BGRA).
fn capture(hwnd: HWND) -> Option<(Vec<u8>, i32, i32)> {
    unsafe {
        let mut r = RECT::default();
        GetWindowRect(hwnd, &mut r).ok()?;
        let (w, h) = (r.right - r.left, r.bottom - r.top);
        if w < 32 || h < 32 || w > 10_000 || h > 10_000 {
            return None;
        }
        let wdc = GetWindowDC(Some(hwnd));
        let mdc = CreateCompatibleDC(Some(wdc));
        let bmp = CreateCompatibleBitmap(wdc, w, h);
        let old = SelectObject(mdc, bmp.into());
        let ok = PrintWindow(hwnd, mdc, PRINT_WINDOW_FLAGS(2)).as_bool();
        let mut bi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -h,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut buf = vec![0u8; (w * h * 4) as usize];
        GetDIBits(mdc, bmp, 0, h as u32, Some(buf.as_mut_ptr() as *mut _), &mut bi, DIB_RGB_COLORS);
        SelectObject(mdc, old);
        let _ = DeleteObject(bmp.into());
        let _ = DeleteDC(mdc);
        ReleaseDC(Some(hwnd), wdc);
        if !ok {
            return None;
        }
        buf.chunks_exact_mut(4).for_each(|p| p[3] = 255);
        Some((buf, w, h))
    }
}

fn ocr_chars(engine: &OcrEngine, bgra: &[u8], w: i32, h: i32) -> windows::core::Result<usize> {
    let buf = CryptographicBuffer::CreateFromByteArray(bgra)?;
    let bmp = SoftwareBitmap::CreateCopyWithAlphaFromBuffer(&buf, BitmapPixelFormat::Bgra8, w, h, BitmapAlphaMode::Premultiplied)?;
    let r = engine.RecognizeAsync(&bmp)?.join()?;
    let mut n = 0;
    for line in r.Lines()? {
        n += line.Text()?.to_string().chars().count();
    }
    Ok(n)
}

fn webp_kb(bgra: &[u8], w: i32, h: i32) -> Option<f64> {
    let rgba: Vec<u8> = bgra.chunks_exact(4).flat_map(|p| [p[2], p[1], p[0], 255]).collect();
    let mut out = Vec::new();
    image_webp::WebPEncoder::new(&mut out).encode(&rgba, w as u32, h as u32, image_webp::ColorType::Rgba8).ok()?;
    Some(out.len() as f64 / 1024.0)
}

/// Sessão WGC na janela em primeiro plano: conta quadros e área alterada.
struct Wgc {
    device: IDirect3DDevice,
    session: Option<(GraphicsCaptureSession, Direct3D11CaptureFramePool)>,
    frames: Arc<AtomicU64>,
    area: Arc<AtomicU64>,
}

impl Wgc {
    fn new() -> windows::core::Result<Self> {
        let mut dev: Option<ID3D11Device> = None;
        unsafe {
            D3D11CreateDevice(None, D3D_DRIVER_TYPE_HARDWARE, HMODULE::default(), D3D11_CREATE_DEVICE_BGRA_SUPPORT, None, D3D11_SDK_VERSION, Some(&mut dev), None, None)?;
            let dxgi: IDXGIDevice = dev.unwrap().cast()?;
            let device: IDirect3DDevice = CreateDirect3D11DeviceFromDXGIDevice(&dxgi)?.cast()?;
            Ok(Self { device, session: None, frames: Arc::default(), area: Arc::default() })
        }
    }
    /// Troca a janela observada; devolve (quadros, área) acumulados da anterior.
    fn switch(&mut self, hwnd: HWND) -> (u64, u64) {
        if let Some((s, p)) = self.session.take() {
            let _ = s.Close();
            let _ = p.Close();
        }
        let prev = (self.frames.swap(0, Ordering::SeqCst), self.area.swap(0, Ordering::SeqCst));
        let start = || -> windows::core::Result<(GraphicsCaptureSession, Direct3D11CaptureFramePool)> {
            let interop = windows::core::factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()?;
            let item: GraphicsCaptureItem = unsafe { interop.CreateForWindow(hwnd)? };
            let pool = Direct3D11CaptureFramePool::CreateFreeThreaded(&self.device, DirectXPixelFormat::B8G8R8A8UIntNormalized, 2, item.Size()?)?;
            let session = pool.CreateCaptureSession(&item)?;
            session.SetIsCursorCaptureEnabled(false)?;
            let _ = session.SetIsBorderRequired(false);
            let (f, a) = (self.frames.clone(), self.area.clone());
            pool.FrameArrived(&TypedEventHandler::<Direct3D11CaptureFramePool, IInspectable>::new(move |p, _| {
                if let Some(p) = p.as_ref()
                    && let Ok(frame) = p.TryGetNextFrame()
                {
                    f.fetch_add(1, Ordering::Relaxed);
                    if let Ok(v) = frame.DirtyRegions() {
                        for r in v {
                            a.fetch_add(r.Width.max(0) as u64 * r.Height.max(0) as u64, Ordering::Relaxed);
                        }
                    }
                    let _ = frame.Close();
                }
                Ok(())
            }))?;
            session.StartCapture()?;
            Ok((session, pool))
        };
        self.session = start().ok();
        prev
    }
}

fn proc_stats() -> (f64, f64, f64) {
    unsafe {
        let (mut a, mut b, mut k, mut u) = (FILETIME::default(), FILETIME::default(), FILETIME::default(), FILETIME::default());
        let _ = GetProcessTimes(GetCurrentProcess(), &mut a, &mut b, &mut k, &mut u);
        let f = |x: FILETIME| ((x.dwHighDateTime as u64) << 32 | x.dwLowDateTime as u64) as f64 / 1e7;
        let mut pmc = PROCESS_MEMORY_COUNTERS { cb: std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32, ..Default::default() };
        let _ = GetProcessMemoryInfo(GetCurrentProcess(), &mut pmc, pmc.cb);
        (f(k) + f(u), pmc.PeakWorkingSetSize as f64 / 1e6, pmc.WorkingSetSize as f64 / 1e6)
    }
}

fn out_dir() -> std::path::PathBuf {
    let base = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".into());
    let d = std::path::Path::new(&base).join("aura-spikes").join("s3");
    let _ = std::fs::create_dir_all(&d);
    d
}

fn save(path: &std::path::Path, start_unix: u64, t0: Instant, apps: &BTreeMap<String, AppStats>) {
    let (cpu, peak, now) = proc_stats();
    let g = Global {
        inicio_unix: start_unix,
        segundos: t0.elapsed().as_secs_f64(),
        ev_foreground: EV_FG.load(Ordering::Relaxed),
        ev_namechange_titulo: EV_TITLE.load(Ordering::Relaxed),
        ev_namechange_total: EV_NAME.load(Ordering::Relaxed),
        cpu_s: cpu,
        ws_pico_mb: peak,
        ws_atual_mb: now,
    };
    let _ = std::fs::write(path, serde_json::to_string_pretty(&report(&g, apps)).unwrap());
}

/// Processa um Momento da janela em primeiro plano (só números).
#[allow(clippy::too_many_arguments)]
fn momento(
    hwnd: HWND, apps: &mut BTreeMap<String, AppStats>, engine: &OcrEngine, uia_tx: &Sender<isize>,
    uia_rx: &Receiver<(usize, f64)>, uia_busy: &mut bool, heartbeat: bool,
) {
    let exe = exe_of(hwnd);
    if exe == "s3-momentos.exe" {
        return;
    }
    let s = apps.entry(exe).or_default();
    if heartbeat {
        s.batidas += 1;
    }
    let mut route = Route::Ocr;
    if !goes_to_ocr(&class_of(hwnd)) {
        // Resultado atrasado de uma leitura anterior: libera a thread UIA.
        if *uia_busy && uia_rx.try_recv().is_ok() {
            *uia_busy = false;
        }
        if *uia_busy {
            route = Route::UiaTimeoutOcr;
        } else {
            let _ = uia_tx.send(hwnd.0 as isize);
            match uia_rx.recv_timeout(UIA_BUDGET) {
                Ok((chars, ms)) => {
                    s.uia_ms.push(ms);
                    s.uia_chars.push(chars as f64);
                    route = if chars == 0 { Route::UiaEmptyOcr } else { Route::Uia };
                }
                Err(_) => {
                    *uia_busy = true;
                    route = Route::UiaTimeoutOcr;
                }
            }
        }
    }
    s.route(route);
    if route == Route::Uia {
        return;
    }
    let t = Instant::now();
    let Some((bgra, w, h)) = capture(hwnd) else { return };
    s.captura_ms.push(t.elapsed().as_secs_f64() * 1000.0);
    s.largura_px.push(w as f64);
    let t = Instant::now();
    if let Ok(n) = ocr_chars(engine, &bgra, w, h) {
        s.ocr_ms.push(t.elapsed().as_secs_f64() * 1000.0);
        s.ocr_chars.push(n as f64);
    }
    let t = Instant::now();
    if let Some(kb) = webp_kb(&bgra, w, h) {
        s.webp_ms.push(t.elapsed().as_secs_f64() * 1000.0);
        s.webp_kb.push(kb);
    }
}

fn rodar(hours: f64) -> anyhow::Result<()> {
    let start_unix = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs();
    let dir = out_dir();
    let path = dir.join(format!("resumo-{start_unix}.json"));
    let stop_file = dir.join("parar");
    let _ = std::fs::remove_file(&stop_file);
    println!("Gravando só números em {}\nPara parar: Ctrl+C, feche esta janela ou crie o arquivo {}", path.display(), stop_file.display());
    unsafe {
        let _ = SetConsoleCtrlHandler(Some(ctrl), true);
    }
    let (tx, rx) = channel::<isize>();
    TX.set(Mutex::new(tx)).ok();
    let limit = Duration::from_secs_f64(hours * 3600.0);
    let t0 = Instant::now();

    // Thread de UIA (MTA), uma leitura por vez.
    let (uia_tx, uia_in) = channel::<isize>();
    let (uia_out, uia_rx) = channel::<(usize, f64)>();
    std::thread::spawn(move || unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let uia: IUIAutomation = CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER).unwrap();
        while let Ok(h) = uia_in.recv() {
            let t = Instant::now();
            let n = uia_chars(&uia, HWND(h as *mut _));
            let _ = uia_out.send((n, t.elapsed().as_secs_f64() * 1000.0));
        }
    });

    // Thread de assentamento, Momentos, WGC e gravação periódica.
    let worker = std::thread::spawn(move || -> anyhow::Result<()> {
        unsafe {
            windows::Win32::System::WinRT::RoInitialize(windows::Win32::System::WinRT::RO_INIT_MULTITHREADED)?;
        }
        let engine = OcrEngine::TryCreateFromLanguage(&Language::CreateLanguage(&HSTRING::from("pt-BR"))?)?;
        let mut wgc = Wgc::new().ok();
        let mut apps: BTreeMap<String, AppStats> = BTreeMap::new();
        let (mut pending, mut last_momento, mut last_save) = (None::<Instant>, Instant::now(), Instant::now());
        let mut uia_busy = false;
        let mut current = HWND::default();
        let mut current_exe = String::new();
        let mut changed_since = 0u64;
        loop {
            match rx.recv_timeout(Duration::from_millis(250)) {
                Ok(_) => pending = Some(Instant::now()),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(_) => break,
            }
            if STOP.load(Ordering::SeqCst) || t0.elapsed() >= limit || stop_file.exists() {
                break;
            }
            let fg = unsafe { GetForegroundWindow() };
            if fg != current && !fg.is_invalid() {
                if let Some(w) = wgc.as_mut() {
                    let (f, a) = w.switch(fg);
                    if !current_exe.is_empty() {
                        let s = apps.entry(current_exe.clone()).or_default();
                        s.wgc_quadros += f;
                        s.wgc_area_px += a as f64;
                    }
                }
                current = fg;
                current_exe = exe_of(fg);
                changed_since = 0;
            }
            let frames_now = wgc.as_ref().map_or(0, |w| w.frames.load(Ordering::Relaxed));
            if let Some(t) = pending
                && t.elapsed() >= SETTLE
            {
                momento(fg, &mut apps, &engine, &uia_tx, &uia_rx, &mut uia_busy, false);
                pending = None;
                last_momento = Instant::now();
                changed_since = frames_now;
            } else if pending.is_none() && last_momento.elapsed() >= HEARTBEAT && frames_now > changed_since {
                // Batida: a tela mudou (WGC) sem evento de janela.
                momento(fg, &mut apps, &engine, &uia_tx, &uia_rx, &mut uia_busy, true);
                last_momento = Instant::now();
                changed_since = frames_now;
            }
            if last_save.elapsed() >= SAVE_EVERY {
                save(&path, start_unix, t0, &apps);
                last_save = Instant::now();
            }
        }
        if let Some(w) = wgc.as_mut() {
            let (f, a) = w.switch(HWND::default());
            let s = apps.entry(current_exe).or_default();
            s.wgc_quadros += f;
            s.wgc_area_px += a as f64;
        }
        save(&path, start_unix, t0, &apps);
        println!("Resumo final: {}", path.display());
        Ok(())
    });

    unsafe {
        let flags = WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS;
        let h1 = SetWinEventHook(EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_FOREGROUND, None, Some(hook), 0, 0, flags);
        let h2 = SetWinEventHook(EVENT_OBJECT_NAMECHANGE, EVENT_OBJECT_NAMECHANGE, None, Some(hook), 0, 0, flags);
        let mut msg = MSG::default();
        while !worker.is_finished() {
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            let _ = MsgWaitForMultipleObjects(None, false, 200, QS_ALLINPUT);
        }
        let _ = UnhookWinEvent(h1);
        let _ = UnhookWinEvent(h2);
    }
    worker.join().map_err(|_| anyhow::anyhow!("worker"))??;
    Ok(())
}

fn find_window(part: &str) -> Option<HWND> {
    unsafe {
        let mut h = GetTopWindow(None).ok()?;
        loop {
            let mut b = [0u16; 512];
            let n = GetWindowTextW(h, &mut b);
            if n > 0 && String::from_utf16_lossy(&b[..n as usize]).contains(part) && IsWindowVisible(h).as_bool() {
                return Some(h);
            }
            h = GetWindow(h, GW_HWNDNEXT).ok()?;
        }
    }
}

fn main() -> anyhow::Result<()> {
    let a: Vec<String> = std::env::args().collect();
    match a.get(1).map(String::as_str) {
        Some("rodar") => rodar(a.get(2).and_then(|h| h.parse().ok()).unwrap_or(10.0)),
        Some(cmd @ ("uia-nome" | "uia-ler")) => unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let uia: IUIAutomation = CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)?;
            let h = find_window(&a[2]).ok_or_else(|| anyhow::anyhow!("janela não encontrada"))?;
            let t = Instant::now();
            if cmd == "uia-nome" {
                let n = uia.ElementFromHandle(h)?.CurrentName()?.to_string().chars().count();
                println!("{{\"caracteres_nome\":{n},\"ms\":{:.1}}}", t.elapsed().as_secs_f64() * 1000.0);
            } else {
                let n = uia_chars(&uia, h);
                println!("{{\"caracteres\":{n},\"ms\":{:.1}}}", t.elapsed().as_secs_f64() * 1000.0);
            }
            Ok(())
        },
        _ => {
            eprintln!("uso: s3-momentos rodar [horas] | uia-nome \"<título>\" | uia-ler \"<título>\"");
            Ok(())
        }
    }
}
