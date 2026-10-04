# QA helper (TK-035): a plain text box the Overlay inserts into. Writes its
# text to $args[0] continuously; closes after $args[1] seconds.
param([string]$Out, [int]$Seconds = 120)
Add-Type -AssemblyName System.Windows.Forms
Add-Type @"
using System; using System.Runtime.InteropServices;
public static class QaFg {
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  // A synthetic ALT press lets this process take the foreground.
  public static void Take(IntPtr h) { keybd_event(0x12, 0, 0, UIntPtr.Zero); keybd_event(0x12, 0, 2, UIntPtr.Zero); SetForegroundWindow(h); }
}
"@
$form = New-Object System.Windows.Forms.Form
$form.Text = "QA Insert Target"
$form.Width = 500; $form.Height = 300; $form.TopMost = $true
$box = New-Object System.Windows.Forms.TextBox
$box.Multiline = $true; $box.Dock = "Fill"
$form.Controls.Add($box)
$timer = New-Object System.Windows.Forms.Timer
$timer.Interval = 200
$start = Get-Date
$timer.Add_Tick({
  [IO.File]::WriteAllText($Out, $box.Text)
  if (((Get-Date) - $start).TotalSeconds -gt $Seconds) { $form.Close() }
})
$form.Add_Shown({ [QaFg]::Take($form.Handle); $form.Activate(); $box.Focus(); $timer.Start() })
[void]$form.ShowDialog()
