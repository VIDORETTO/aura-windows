// QA-041 / 001 AC-001: starting Aura shows no window (tray only); starting
// the executable again while it runs opens the Overlay (single instance).
// Window visibility is read from Windows itself (EnumWindows), not the DOM.
import { spawn, spawnSync } from "node:child_process";
import path from "node:path";

const app = process.env.AURA_E2E_APP!;
const exe = path.basename(app, ".exe");
const VISIBLE = `
Add-Type @"
using System; using System.Runtime.InteropServices; using System.Text;
public static class QaWin {
  public delegate bool Cb(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] public static extern bool EnumWindows(Cb cb, IntPtr l);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [StructLayout(LayoutKind.Sequential)] public struct R { public int L, T, Rt, B; }
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out R r);
  [DllImport("dwmapi.dll")] public static extern int DwmGetWindowAttribute(IntPtr h, int a, out int v, int s);
}
"@
$ids = @(Get-Process -Name '${exe}' -ErrorAction SilentlyContinue | ForEach-Object { $_.Id })
$titles = New-Object System.Collections.Generic.List[string]
[QaWin]::EnumWindows({ param($h, $l) $p = 0; [void][QaWin]::GetWindowThreadProcessId($h, [ref]$p)
  if ($ids -contains [int]$p -and [QaWin]::IsWindowVisible($h)) {
    $r = New-Object QaWin+R; [void][QaWin]::GetWindowRect($h, [ref]$r)
    $cloaked = 0; [void][QaWin]::DwmGetWindowAttribute($h, 14, [ref]$cloaked, 4)
    # Only windows a person can see: larger than the 16x16 helper windows of
    # the tray and single-instance plugins, and not cloaked by DWM.
    if (($r.Rt - $r.L) -gt 32 -and ($r.B - $r.T) -gt 32 -and $cloaked -eq 0) {
      $s = New-Object Text.StringBuilder 256; [void][QaWin]::GetWindowText($h, $s, 256)
      $titles.Add("$($s.ToString()) $($r.Rt - $r.L)x$($r.B - $r.T)")
    }
  }
  return $true }, [IntPtr]::Zero) | Out-Null
$titles -join "|"
`;
const visibleWindows = () => spawnSync("powershell", ["-NoProfile", "-Command", VISIBLE], { encoding: "utf8" }).stdout.trim();

describe("Startup in the tray (QA-041)", () => {
  it("starts without windows and opens the Overlay when launched again", async () => {
    await $("body").waitForExist({ timeout: 30_000 });
    await browser.pause(2000);
    const atStart = visibleWindows();
    console.log("Native visible windows at start", JSON.stringify(atStart));
    expect(atStart).toBe("");
    const second = spawn(app, [], { stdio: "ignore", env: process.env });
    await browser.waitUntil(() => visibleWindows() !== "", { timeout: 20_000, timeoutMsg: "second launch did not open the Overlay" });
    console.log("Native visible windows after relaunch", JSON.stringify(visibleWindows()));
    second.unref();
  });
});
