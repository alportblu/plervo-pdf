<# Native window diagnostics on a disposable Windows CI desktop, never the user's computer. #>
param([string] $Executable, [string[]] $Arguments, [string] $Folder, [string] $ExpectedTitle)
$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_OS -ne 'Windows') { throw 'Disposable Windows GitHub runner required' }
Add-Type -AssemblyName System.Drawing, System.Windows.Forms
if (-not ('PlervoWindow' -as [type])) {
Add-Type @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
public static class PlervoWindow {
  [StructLayout(LayoutKind.Sequential)] public struct Rect { public int Left, Top, Right, Bottom; }
  [StructLayout(LayoutKind.Sequential)] public struct Point { public int X, Y; }
  public sealed class WindowInfo { public long Handle; public string Title; public string ClassName; public int Width; public int Height; }
  private delegate bool EnumProc(IntPtr window, IntPtr parameter);
  [DllImport("user32.dll")] private static extern bool EnumWindows(EnumProc callback, IntPtr parameter);
  [DllImport("user32.dll")] private static extern bool IsWindowVisible(IntPtr window);
  [DllImport("user32.dll")] private static extern uint GetWindowThreadProcessId(IntPtr window, out uint process);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] private static extern int GetWindowText(IntPtr window, StringBuilder text, int count);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] private static extern int GetClassName(IntPtr window, StringBuilder text, int count);
  [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)] public static extern bool PostMessage(IntPtr window, uint message, IntPtr wParam, IntPtr lParam);
  [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)] private static extern IntPtr SendMessageTimeout(IntPtr window, uint message, IntPtr wParam, IntPtr lParam, uint flags, uint timeout, out IntPtr result);
  [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr window, out Rect rect);
  [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr window, ref Point point);
  [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr window, IntPtr after, int x, int y, int width, int height, uint flags);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr window);
  [DllImport("user32.dll")] public static extern bool ShowWindowAsync(IntPtr window, int command);
  public static bool Responding(IntPtr window) { IntPtr result; return SendMessageTimeout(window, 0, IntPtr.Zero, IntPtr.Zero, 2, 5000, out result) != IntPtr.Zero; }
  public static WindowInfo[] ForProcess(uint process) {
    var windows = new List<WindowInfo>();
    EnumWindows((window, parameter) => {
      uint owner; GetWindowThreadProcessId(window, out owner);
      if (owner != process || !IsWindowVisible(window)) return true;
      var title = new StringBuilder(512); var name = new StringBuilder(256); Rect bounds;
      GetWindowText(window, title, title.Capacity); GetClassName(window, name, name.Capacity);
      if (GetClientRect(window, out bounds)) windows.Add(new WindowInfo { Handle = window.ToInt64(), Title = title.ToString(), ClassName = name.ToString(), Width = bounds.Right - bounds.Left, Height = bounds.Bottom - bounds.Top });
      return true;
    }, IntPtr.Zero);
    return windows.ToArray();
  }
}
'@
}
New-Item -ItemType Directory -Force -Path $Folder | Out-Null
$App = $null
$Result = [ordered] @{ passed = $false; normal_close = $false; scope = 'Installed GUI launch, response, app-only capture, no TCP listener and normal close. Screenshots require visual review.' }
try {
  $Start = @{ FilePath = $Executable; PassThru = $true; RedirectStandardOutput = (Join-Path $Folder 'gui-stdout.txt'); RedirectStandardError = (Join-Path $Folder 'gui-stderr.txt') }
  if ($Arguments.Count) { $Start.ArgumentList = $Arguments }
  $App = Start-Process @Start
    # Retain the process handle while it is alive so Windows preserves its exit code.
    [void] $App.Handle
    $Deadline = [DateTime]::UtcNow.AddSeconds(90)
    $Selected = $null
    $Ready = $false
    do {
      Start-Sleep -Milliseconds 500
      $App.Refresh()
      if ($App.HasExited) { throw "Application exited before opening a window: $($App.ExitCode)" }
      # Winit exposes an untitled event-target helper alongside the actual app.
      # Enumerate only this process and require its expected, rendered window.
      $Windows = @([PlervoWindow]::ForProcess($App.Id))
      $Selected = $Windows | Where-Object { $_.Title.IndexOf($ExpectedTitle, [StringComparison]::Ordinal) -ge 0 -and $_.Width -ge 320 -and $_.Height -ge 240 } | Sort-Object { $_.Width * $_.Height } -Descending | Select-Object -First 1
      # A titled window may exist while the software renderer is still starting.
      $Ready = $Selected -and [PlervoWindow]::Responding([IntPtr] $Selected.Handle)
    } while (-not $Ready -and [DateTime]::UtcNow -lt $Deadline)
    ConvertTo-Json -InputObject $Windows -Depth 5 | Set-Content -LiteralPath (Join-Path $Folder 'observed-windows.json') -Encoding utf8
    if (-not $Ready) { throw 'No responsive expected application window within 90 seconds' }
    $Window = [IntPtr] $Selected.Handle
    $Screen = [Windows.Forms.Screen]::PrimaryScreen.WorkingArea
    Write-Host "$ExpectedTitle : native window found; checking responsiveness"
    if (-not [PlervoWindow]::Responding($Window)) { throw 'Application window is not responding after startup' }
    # Cross-thread window requests must not wait forever on a hung application.
    # https://learn.microsoft.com/windows/win32/api/winuser/nf-winuser-setwindowpos
    [void] [PlervoWindow]::ShowWindowAsync($Window, 9)
    if (-not [PlervoWindow]::SetWindowPos($Window, [IntPtr]::Zero, $Screen.X, $Screen.Y, [Math]::Min(1280, $Screen.Width), [Math]::Min(800, $Screen.Height - 40), 0x4000)) { throw 'Cannot position the CI test window' }
    [void] [PlervoWindow]::SetForegroundWindow($Window)
    Start-Sleep -Seconds 5
    $App.Refresh()
    if ($App.HasExited -or -not [PlervoWindow]::Responding($Window)) { throw 'Application window exited or stopped responding' }
    $Selected = @([PlervoWindow]::ForProcess($App.Id)) | Where-Object { $_.Handle -eq $Window.ToInt64() -and ($_.Title.IndexOf($ExpectedTitle, [StringComparison]::Ordinal) -ge 0) } | Select-Object -First 1
    if (-not $Selected) { throw 'Selected application window changed before capture' }
    $Rect = [PlervoWindow+Rect]::new()
    $Origin = [PlervoWindow+Point]::new()
    if (-not [PlervoWindow]::GetClientRect($Window, [ref] $Rect) -or -not [PlervoWindow]::ClientToScreen($Window, [ref] $Origin)) { throw 'Cannot inspect the application client area' }
    # Preserve the app's own rendered UI, excluding the OS title bar and desktop artwork.
    $Bounds = [Drawing.Rectangle]::new($Origin.X, $Origin.Y, $Rect.Right - $Rect.Left, $Rect.Bottom - $Rect.Top)
    $Capture = [Drawing.Rectangle]::Intersect($Bounds, $Screen)
    if ($Capture.Width -lt 320 -or $Capture.Height -lt 240) { throw 'Application window is missing or too small' }
    $Bitmap = [Drawing.Bitmap]::new($Capture.Width, $Capture.Height)
    $Graphics = [Drawing.Graphics]::FromImage($Bitmap)
    try {
      $Graphics.CopyFromScreen($Capture.Location, [Drawing.Point]::Empty, $Capture.Size)
      $Bitmap.Save((Join-Path $Folder 'first-launch.png'), [Drawing.Imaging.ImageFormat]::Png)
    } finally { $Graphics.Dispose(); $Bitmap.Dispose() }
    $Listeners = @(Get-NetTCPConnection -State Listen -ErrorAction SilentlyContinue | Where-Object OwningProcess -eq $App.Id)
    if ($Listeners.Count) { throw 'Public app opened a listening TCP endpoint.' }
    $Result.listening_tcp_endpoints = $Listeners.Count
    $Result.window_title = $Selected.Title
    $Result.window_class = $Selected.ClassName
    $Result.window_handle = $Selected.Handle
    $Result.capture_width = $Capture.Width
    $Result.capture_height = $Capture.Height
    $Result.clipped_by_display = ($Bounds.Width -ne $Capture.Width -or $Bounds.Height -ne $Capture.Height)
    Write-Host "$ExpectedTitle : captured client area; requesting normal close"
    # Request a normal close on the exact captured app HWND, never a helper HWND.
    if (-not [PlervoWindow]::PostMessage($Window, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) -or -not $App.WaitForExit(15000)) {
      if (-not $App.HasExited) {
        # Preserve a possible save/quit prompt before the isolated process is stopped.
        try {
          $ClosingImage = [Drawing.Bitmap]::new($Capture.Width, $Capture.Height)
          $ClosingGraphics = [Drawing.Graphics]::FromImage($ClosingImage)
          try {
            $ClosingGraphics.CopyFromScreen($Capture.Location, [Drawing.Point]::Empty, $Capture.Size)
            $ClosingImage.Save((Join-Path $Folder 'close-request.png'), [Drawing.Imaging.ImageFormat]::Png)
          } finally { $ClosingGraphics.Dispose(); $ClosingImage.Dispose() }
        } catch { Write-Warning "Could not capture the close failure: $($_.ToString())" }
      }
      throw 'Application did not close normally'
    }
    $Result.exit_code = $App.ExitCode
    if ($null -eq $Result.exit_code -or $Result.exit_code -ne 0) { throw "Application closed with error: $($Result.exit_code)" }
    $Result.normal_close = $true
  $Result.passed = $true
} catch {
  $Result.error = $_.ToString()
  throw
} finally {
  if ($App -and -not $App.HasExited) { $App.Kill(); [void] $App.WaitForExit(10000) }
  if ($App) { $App.Dispose() }
  $Result | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $Folder 'gui-result.json') -Encoding utf8
}
