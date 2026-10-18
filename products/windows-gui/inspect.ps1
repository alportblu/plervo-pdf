<# Native window diagnostics on a disposable Windows CI desktop, never the user's computer. #>
$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_OS -ne 'Windows') { throw 'Disposable Windows GitHub runner required' }
Add-Type -AssemblyName System.Drawing, System.Windows.Forms
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
$Root = (Resolve-Path 'dist/windows-gui').Path
$Rows = Get-Content -Raw -LiteralPath (Join-Path $Root 'selected.json') | ConvertFrom-Json
$Results = [Collections.Generic.List[object]]::new()
function Invoke-Msi([string] $Msi, [string] $Folder, [string] $Operation, [string] $Label) {
  $Log = Join-Path $Folder "$Label-msi.log"
  $Info = [Diagnostics.ProcessStartInfo]::new((Join-Path $env:SystemRoot 'System32/msiexec.exe'))
  $Info.UseShellExecute = $false
  $Info.CreateNoWindow = $true
  foreach ($Argument in @($Operation, $Msi, '/qn', '/norestart', '/L*v', $Log)) { $Info.ArgumentList.Add($Argument) }
  $Job = [Diagnostics.Process]::new()
  $Job.StartInfo = $Info
  try {
    if (-not $Job.Start()) { throw "Cannot start MSI $Label" }
    if (-not $Job.WaitForExit(180000)) { $Job.Kill(); throw "MSI $Label timed out" }
    if ($Job.ExitCode -notin @(0, 3010)) {
      Select-String -LiteralPath $Log -Pattern 'Return value 3|Error [0-9]{4}' -Context 8,3 | Select-Object -Last 5 | Out-Host
      throw "MSI $Label failed: $($Job.ExitCode)"
    }
  } finally { $Job.Dispose() }
}
foreach ($Row in $Rows) {
  if ($Row.product -notin @('pdf','photo','vector','video','develop','motion','publish','write','slides','sheets','cad','audio','canvas')) { throw 'Unknown product' }
  $Name = if ($Row.product -in @('pdf','cad')) { 'Plervo ' + $Row.product.ToUpperInvariant() } else { 'Plervo ' + [cultureinfo]::InvariantCulture.TextInfo.ToTitleCase($Row.product) }
  if ($Row.name -cne $Name -or $Row.msi -cne "plervo-$($Row.product)-0.1.0-alpha.4-windows-x64.msi") { throw 'Unexpected candidate identity' }
  $Folder = Join-Path $Root $Row.product
  $Msi = Join-Path $Folder $Row.msi
  if ((Get-FileHash -LiteralPath $Msi -Algorithm SHA256).Hash.ToLowerInvariant() -cne $Row.msi_sha256) { throw 'Candidate MSI changed after verification' }
  $Install = Join-Path $env:ProgramFiles $Name
  if (Test-Path -LiteralPath $Install) { throw 'Refusing to replace an existing application' }
  $Installed = $false
  $App = $null
  $Failure = $null
  $Result = [ordered] @{ product = $Row.product; native_run = $Row.run; native_head = $Row.head; msi_sha256 = $Row.msi_sha256; passed = $false; signed = $false; public_qualification = $false; scope = 'Native startup, window capture and normal close. Screenshot requires visual review; no complete feature or hardware acceptance.' }
  try {
    Write-Host "$Name : installing exact MSI"
    Invoke-Msi $Msi $Folder '/i' 'install'
    $Installed = $true
    $Binary = Join-Path $Install "plervo-$($Row.product).exe"
    $Result.binary_sha256 = (Get-FileHash -LiteralPath $Binary -Algorithm SHA256).Hash.ToLowerInvariant()
    Write-Host "$Name : starting installed app"
    # No token is passed to this step. The app has no user document to open or modify.
    $App = Start-Process -FilePath $Binary -PassThru -RedirectStandardOutput (Join-Path $Folder 'gui-stdout.txt') -RedirectStandardError (Join-Path $Folder 'gui-stderr.txt')
    $Deadline = [DateTime]::UtcNow.AddSeconds(90)
    $Selected = $null
    $Ready = $false
    do {
      Start-Sleep -Milliseconds 500
      $App.Refresh()
      if ($App.HasExited) { throw "Application exited before opening a window: $($App.ExitCode)" }
      # Winit exposes an untitled event-target helper alongside the actual app.
      # Enumerate only this process and require its branded, rendered window.
      $Windows = @([PlervoWindow]::ForProcess($App.Id))
      $Selected = $Windows | Where-Object { $_.Title.Contains($Name, [StringComparison]::Ordinal) -and $_.Width -ge 320 -and $_.Height -ge 240 } | Sort-Object { $_.Width * $_.Height } -Descending | Select-Object -First 1
      # A titled window may exist while the software renderer is still starting.
      $Ready = $Selected -and [PlervoWindow]::Responding([IntPtr] $Selected.Handle)
    } while (-not $Ready -and [DateTime]::UtcNow -lt $Deadline)
    ConvertTo-Json -InputObject $Windows -Depth 5 | Set-Content -LiteralPath (Join-Path $Folder 'observed-windows.json') -Encoding utf8
    if (-not $Ready) { throw 'No responsive branded application window within 90 seconds' }
    $Window = [IntPtr] $Selected.Handle
    $Screen = [Windows.Forms.Screen]::PrimaryScreen.WorkingArea
    Write-Host "$Name : native window found; checking responsiveness"
    if (-not [PlervoWindow]::Responding($Window)) { throw 'Application window is not responding after startup' }
    # Cross-thread window requests must not wait forever on a hung application.
    # https://learn.microsoft.com/windows/win32/api/winuser/nf-winuser-setwindowpos
    [void] [PlervoWindow]::ShowWindowAsync($Window, 9)
    if (-not [PlervoWindow]::SetWindowPos($Window, [IntPtr]::Zero, $Screen.X, $Screen.Y, [Math]::Min(1280, $Screen.Width), [Math]::Min(800, $Screen.Height - 40), 0x4000)) { throw 'Cannot position the CI test window' }
    [void] [PlervoWindow]::SetForegroundWindow($Window)
    Start-Sleep -Seconds 5
    $App.Refresh()
    if ($App.HasExited -or -not [PlervoWindow]::Responding($Window)) { throw 'Application window exited or stopped responding' }
    $Selected = @([PlervoWindow]::ForProcess($App.Id)) | Where-Object { $_.Handle -eq $Window.ToInt64() -and $_.Title.Contains($Name, [StringComparison]::Ordinal) } | Select-Object -First 1
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
    $Result.window_title = $Selected.Title
    $Result.window_class = $Selected.ClassName
    $Result.window_handle = $Selected.Handle
    $Result.capture_width = $Capture.Width
    $Result.capture_height = $Capture.Height
    $Result.clipped_by_display = ($Bounds.Width -ne $Capture.Width -or $Bounds.Height -ne $Capture.Height)
    Write-Host "$Name : captured client area; requesting normal close"
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
    if ($App.ExitCode -ne 0) { throw "Application closed with error: $($App.ExitCode)" }
    $Result.normal_close = $true
  } catch { $Failure = $_.ToString() }
  finally {
    if ($App -and -not $App.HasExited) { $App.Kill($true); [void] $App.WaitForExit(10000) }
    if ($Installed) {
      try {
        Write-Host "$Name : removing isolated CI installation"
        Invoke-Msi $Msi $Folder '/x' 'uninstall'
        if (Test-Path -LiteralPath (Join-Path $Install "plervo-$($Row.product).exe")) { throw 'Uninstall left the executable' }
      } catch { $Failure = "$Failure Uninstall: $($_.ToString())" }
    }
  }
  $Result.passed = -not $Failure
  if ($Failure) { $Result.error = $Failure; Write-Warning "$Name : $Failure" }
  $Result | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $Folder 'gui-result.json') -Encoding utf8
  $Results.Add($Result)
  ConvertTo-Json -InputObject $Results -Depth 5 | Set-Content -LiteralPath (Join-Path $Root 'summary.json') -Encoding utf8
  Write-Host "$Name : diagnostics complete; passed=$($Result.passed)"
}
ConvertTo-Json -InputObject $Results -Depth 5 | Set-Content -LiteralPath (Join-Path $Root 'summary.json') -Encoding utf8
if (@($Results | Where-Object { -not $_.passed }).Count) { throw 'One or more native GUI checks failed; all available diagnostics were preserved.' }
