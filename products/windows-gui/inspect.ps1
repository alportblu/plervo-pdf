<# Native window diagnostics on a disposable Windows CI desktop, never the user's computer. #>
$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_OS -ne 'Windows') { throw 'Disposable Windows GitHub runner required' }
Add-Type -AssemblyName System.Drawing, System.Windows.Forms
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class PlervoWindow {
  [StructLayout(LayoutKind.Sequential)] public struct Rect { public int Left, Top, Right, Bottom; }
  [StructLayout(LayoutKind.Sequential)] public struct Point { public int X, Y; }
  [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr window, out Rect rect);
  [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr window, ref Point point);
  [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr window, IntPtr after, int x, int y, int width, int height, uint flags);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr window);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr window, int command);
}
'@
$Root = (Resolve-Path 'dist/windows-gui').Path
$Rows = Get-Content -Raw -LiteralPath (Join-Path $Root 'selected.json') | ConvertFrom-Json
$Results = [Collections.Generic.List[object]]::new()
function Invoke-Msi([string] $Msi, [string] $Folder, [string] $Operation, [string] $Label) {
  $Log = Join-Path $Folder "$Label-msi.log"
  $Job = Start-Process msiexec.exe -ArgumentList @($Operation, "`"$Msi`"", '/qn', '/norestart', '/L*v', "`"$Log`"") -PassThru
  if (-not $Job.WaitForExit(180000)) { $Job.Kill($true); throw "MSI $Label timed out" }
  if ($Job.ExitCode -notin @(0, 3010)) { throw "MSI $Label failed: $($Job.ExitCode)" }
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
    Invoke-Msi $Msi $Folder '/i' 'install'
    $Installed = $true
    $Binary = Join-Path $Install "plervo-$($Row.product).exe"
    $Result.binary_sha256 = (Get-FileHash -LiteralPath $Binary -Algorithm SHA256).Hash.ToLowerInvariant()
    # No token is passed to this step. The app has no user document to open or modify.
    $App = Start-Process -FilePath $Binary -PassThru -RedirectStandardOutput (Join-Path $Folder 'gui-stdout.txt') -RedirectStandardError (Join-Path $Folder 'gui-stderr.txt')
    $Deadline = [DateTime]::UtcNow.AddSeconds(90)
    do {
      Start-Sleep -Milliseconds 500
      $App.Refresh()
      if ($App.HasExited) { throw "Application exited before opening a window: $($App.ExitCode)" }
    } while ($App.MainWindowHandle -eq [IntPtr]::Zero -and [DateTime]::UtcNow -lt $Deadline)
    if ($App.MainWindowHandle -eq [IntPtr]::Zero) { throw 'No application window within 90 seconds' }
    $Window = $App.MainWindowHandle
    $Screen = [Windows.Forms.Screen]::PrimaryScreen.WorkingArea
    [void] [PlervoWindow]::ShowWindow($Window, 9)
    if (-not [PlervoWindow]::SetWindowPos($Window, [IntPtr]::Zero, $Screen.X, $Screen.Y, [Math]::Min(1280, $Screen.Width), [Math]::Min(800, $Screen.Height - 40), 0)) { throw 'Cannot position the CI test window' }
    [void] [PlervoWindow]::SetForegroundWindow($Window)
    Start-Sleep -Seconds 5
    $App.Refresh()
    if ($App.HasExited -or -not $App.Responding) { throw 'Application window exited or stopped responding' }
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
    $Result.window_title = $App.MainWindowTitle
    $Result.capture_width = $Capture.Width
    $Result.capture_height = $Capture.Height
    $Result.clipped_by_display = ($Bounds.Width -ne $Capture.Width -or $Bounds.Height -ne $Capture.Height)
    if (-not $App.CloseMainWindow() -or -not $App.WaitForExit(15000)) { throw 'Application did not close normally' }
    if ($App.ExitCode -ne 0) { throw "Application closed with error: $($App.ExitCode)" }
    $Result.normal_close = $true
  } catch { $Failure = $_.ToString() }
  finally {
    if ($App -and -not $App.HasExited) { $App.Kill($true); [void] $App.WaitForExit(10000) }
    if ($Installed) {
      try {
        Invoke-Msi $Msi $Folder '/x' 'uninstall'
        if (Test-Path -LiteralPath (Join-Path $Install "plervo-$($Row.product).exe")) { throw 'Uninstall left the executable' }
      } catch { $Failure = "$Failure Uninstall: $($_.ToString())" }
    }
  }
  $Result.passed = -not $Failure
  if ($Failure) { $Result.error = $Failure; Write-Warning "$Name : $Failure" }
  $Result | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $Folder 'gui-result.json') -Encoding utf8
  $Results.Add($Result)
}
ConvertTo-Json -InputObject $Results -Depth 5 | Set-Content -LiteralPath (Join-Path $Root 'summary.json') -Encoding utf8
if (@($Results | Where-Object { -not $_.passed }).Count) { throw 'One or more native GUI checks failed; all available diagnostics were preserved.' }
