# QA helper (012): a WPF text box (UI Automation TextPattern, like Notepad,
# Word or a browser) whose text is selected on request; it then gives the
# foreground back to the Aura Overlay, like a user clicking on it.
# Commands are read from $args[0]: "select <text>" or "focus-aura". Results of
# each command are appended to "<commands>.log".
param([string]$Commands, [int]$Seconds = 120, [string]$AuraProcess = "aura")
Add-Type -AssemblyName PresentationFramework, WindowsBase
Add-Type @"
using System; using System.Runtime.InteropServices;
public static class QaSel {
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  public delegate bool Cb(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] public static extern bool EnumWindows(Cb cb, IntPtr l);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [StructLayout(LayoutKind.Sequential)] public struct R { public int L, T, Rt, B; }
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out R r);
  // Largest visible top-level window of the given process (the Overlay).
  public static IntPtr Largest(uint[] pids) {
    IntPtr best = IntPtr.Zero; long area = 0;
    EnumWindows((h, l) => { uint p; GetWindowThreadProcessId(h, out p);
      if (Array.IndexOf(pids, p) >= 0 && IsWindowVisible(h)) { R r; GetWindowRect(h, out r);
        long a = (long)(r.Rt - r.L) * (r.B - r.T); if (a > area) { area = a; best = h; } }
      return true; }, IntPtr.Zero);
    return best;
  }
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  // A synthetic ALT press lets a background process change the foreground.
  public static bool Take(IntPtr h) { keybd_event(0x12, 0, 0, UIntPtr.Zero); keybd_event(0x12, 0, 2, UIntPtr.Zero); return SetForegroundWindow(h); }
}
"@
$log = "$Commands.log"
$window = New-Object System.Windows.Window
$window.Title = "QA Selection Target"; $window.Width = 500; $window.Height = 200; $window.Topmost = $true
$box = New-Object System.Windows.Controls.TextBox
$box.AcceptsReturn = $true; $box.IsInactiveSelectionHighlightEnabled = $true
$window.Content = $box
$script:seen = 0
$start = Get-Date
$timer = New-Object System.Windows.Threading.DispatcherTimer
$timer.Interval = [TimeSpan]::FromMilliseconds(150)
$timer.Add_Tick({
  if (((Get-Date) - $start).TotalSeconds -gt $Seconds) { $window.Close(); return }
  if (-not (Test-Path $Commands)) { return }
  $lines = @(Get-Content -Encoding UTF8 $Commands)
  while ($script:seen -lt $lines.Count) {
    $line = $lines[$script:seen]; $script:seen++
    $handle = (New-Object System.Windows.Interop.WindowInteropHelper $window).Handle
    if ($line -like "select *") {
      $ok = [QaSel]::Take($handle); $window.Activate() | Out-Null; $box.Focus() | Out-Null
      $box.Text = $line.Substring(7); $box.SelectAll()
      Add-Content $log "select foreground=$ok"
    } elseif ($line -eq "focus-aura") {
      $pids = [uint32[]]@(Get-Process -Name $AuraProcess -ErrorAction SilentlyContinue | ForEach-Object { [uint32]$_.Id })
      $aura = [QaSel]::Largest($pids)
      $ok = if ($aura -ne [IntPtr]::Zero) { [QaSel]::Take($aura) } else { $false }
      Add-Content $log "focus-aura found=$($aura -ne [IntPtr]::Zero) foreground=$ok"
    }
  }
})
$window.Add_ContentRendered({
  $handle = (New-Object System.Windows.Interop.WindowInteropHelper $window).Handle
  [QaSel]::Take($handle) | Out-Null
  $timer.Start()
})
[void]$window.ShowDialog()
