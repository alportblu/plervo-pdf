<# Compiled-MSI inspection and lifecycle acceptance on a disposable Windows runner only. #>
param([Parameter(Mandatory)] [string] $Path)
$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_OS -ne 'Windows') {
  throw 'This acceptance test requires a disposable Windows GitHub runner.'
}
$Msi = (Resolve-Path -LiteralPath $Path).Path
$Install = Join-Path $env:ProgramFiles 'Plervo Suite'
if (Test-Path -LiteralPath $Install) { throw 'Refusing to replace an existing Suite installation' }
$Build = Get-Content -Raw -LiteralPath (Join-Path (Split-Path $Msi) 'PLERVO-SUITE-BUILD.json') | ConvertFrom-Json
$ExpectedMsi = @($Build.files | Where-Object file -eq ([IO.Path]::GetFileName($Msi)))
if ($ExpectedMsi.Count -ne 1 -or (Get-FileHash -LiteralPath $Msi -Algorithm SHA256).Hash.ToLowerInvariant() -cne $ExpectedMsi[0].sha256) {
  throw 'MSI differs from its candidate manifest'
}
$Evidence = Join-Path $PWD 'dist/validation/windows-suite'
$Work = Join-Path $env:RUNNER_TEMP ('plervo-suite-acceptance-' + [Guid]::NewGuid())
$Preferences = Join-Path $env:LOCALAPPDATA 'Plervo Suite'
New-Item -ItemType Directory -Force -Path $Evidence, $Work, $Preferences | Out-Null
$Sentinel = Join-Path $Preferences 'acceptance-preserve.txt'
if (Test-Path -LiteralPath $Sentinel) { throw 'Refusing to replace a preference sentinel' }
'Preserve Suite user settings' | Set-Content -LiteralPath $Sentinel
$Document = Join-Path $Work 'document.txt'
'Preserve user documents' | Set-Content -LiteralPath $Document
$ChoicePath = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.pdf\UserChoice'
$ChoiceBefore = (Get-ItemProperty -LiteralPath $ChoicePath -ErrorAction SilentlyContinue).ProgId

$Installer = New-Object -ComObject WindowsInstaller.Installer
$Database = $Installer.OpenDatabase($Msi, 0)
function Read-Value([string] $Sql) {
  $View = $Database.OpenView($Sql)
  try {
    [void] $View.Execute()
    $Row = $View.Fetch()
    if (-not $Row) { throw "Compiled MSI row missing: $Sql" }
    return $Row.StringData(1)
  } finally { [void] $View.Close() }
}
try {
  foreach ($Pair in @(@('ProductName', 'Plervo Suite'), @('Manufacturer', 'Plervo'), @('ALLUSERS', '1'), @('ARPPRODUCTICON', 'PlervoSuite.ico'))) {
    if ((Read-Value ('SELECT `Value` FROM `Property` WHERE `Property` = ''' + $Pair[0] + '''')) -cne $Pair[1]) { throw "Unexpected MSI property $($Pair[0])" }
  }
  if ((Read-Value 'SELECT `FileName` FROM `File` WHERE `File` = ''SuiteExecutable''') -notmatch '(?:^|\|)plervo-suite\.exe$') { throw 'Suite executable missing from MSI' }
  [void] (Read-Value 'SELECT `Name` FROM `Shortcut` WHERE `Shortcut` = ''SuiteStartMenu''')
  $View = $Database.OpenView('SELECT `Control` FROM `Control` WHERE `Control` = ''PdfContextMenu'' OR `Control` = ''DesktopShortcut''')
  try { [void] $View.Execute(); if ($View.Fetch()) { throw 'Suite installer exposes PDF shell options' } } finally { [void] $View.Close() }
  $Condition = Read-Value 'SELECT `Condition` FROM `LaunchCondition`'
  $Installer.UILevel = 2
  $Session = $Installer.OpenPackage($Msi, 1)
  try {
    foreach ($Case in @(@('1', 1), @('2', 0), @('', 0))) {
      $Session.Property('ALLUSERS') = $Case[0]
      if ($Session.EvaluateCondition($Condition) -ne $Case[1]) { throw 'MSI install scope guard failed' }
    }
  } finally { [void] [Runtime.InteropServices.Marshal]::FinalReleaseComObject($Session) }
} finally {
  [void] [Runtime.InteropServices.Marshal]::FinalReleaseComObject($Database)
  [void] [Runtime.InteropServices.Marshal]::FinalReleaseComObject($Installer)
}

function Invoke-Msi([string] $Operation, [string] $Label) {
  $Log = Join-Path $Evidence "$Label-msi.log"
  $Process = Start-Process msiexec.exe -ArgumentList @($Operation, "`"$Msi`"", '/qn', '/norestart', '/L*v', "`"$Log`"") -Wait -PassThru
  if ($Process.ExitCode -notin @(0, 3010)) { throw "Suite MSI $Label failed: $($Process.ExitCode)" }
}
function Invoke-Suite([string[]] $Arguments, [int] $ExpectedExit = 0, [string] $Label, [string] $Executable = (Join-Path $Install 'plervo-suite.exe')) {
  $Start = [Diagnostics.ProcessStartInfo]::new($Executable)
  $Start.UseShellExecute = $false
  $Start.RedirectStandardOutput = $true
  $Start.RedirectStandardError = $true
  foreach ($Argument in $Arguments) { $Start.ArgumentList.Add($Argument) }
  $Process = [Diagnostics.Process]::Start($Start)
  $Output = $Process.StandardOutput.ReadToEndAsync()
  $ErrorText = $Process.StandardError.ReadToEndAsync()
  if (-not $Process.WaitForExit(60000)) { $Process.Kill($true); throw 'Installed Suite command timed out' }
  $Text = $Output.GetAwaiter().GetResult()
  $Errors = $ErrorText.GetAwaiter().GetResult()
  if (-not $Label) { $Label = $Arguments[0].TrimStart('-') }
  $Text | Set-Content -LiteralPath (Join-Path $Evidence "$Label-stdout.txt")
  $Errors | Set-Content -LiteralPath (Join-Path $Evidence "$Label-stderr.txt")
  if ($Process.ExitCode -ne $ExpectedExit) { throw "Installed Suite $Label failed: $($Process.ExitCode): $Errors" }
  return $Text
}
$Installed = $false
try {
  Invoke-Msi '/i' 'install'
  $Installed = $true
  $Binary = Join-Path $Install 'plervo-suite.exe'
  if ((Get-FileHash -LiteralPath $Binary -Algorithm SHA256).Hash.ToLowerInvariant() -cne $Build.binary_sha256) { throw 'Installed Suite binary changed' }
  $Shortcut = Join-Path ([Environment]::GetFolderPath('CommonPrograms')) 'Plervo Suite.lnk'
  if (-not (Test-Path -LiteralPath $Shortcut)) { throw 'Installed Suite has no Start Menu shortcut' }
  if (Test-Path -LiteralPath (Join-Path $Install 'portable.txt')) { throw 'MSI incorrectly enables portable mode' }
  if ((Invoke-Suite -Arguments @('--version')).Trim() -cne "Plervo Suite $($Build.version)") { throw 'Unexpected installed version' }
  if ((Invoke-Suite -Arguments @('--status', 'pdf')).Trim() -cne 'null') { throw 'Clean runner unexpectedly has a managed PDF installation' }
  [void] (Invoke-Suite -Arguments @('--status', '../other') -ExpectedExit 1 -Label 'status-invalid')
  $Capture = Join-Path $Evidence 'suite-native.png'
  [void] (Invoke-Suite -Arguments @('--native-smoke-test', $Capture))
  if (-not (Test-Path -LiteralPath $Capture) -or (Get-Item -LiteralPath $Capture).Length -lt 1000) { throw 'Native Suite rendering was not captured' }
  Remove-Item -LiteralPath $Binary
  Invoke-Msi '/fa' 'repair'
  if ((Get-FileHash -LiteralPath $Binary -Algorithm SHA256).Hash.ToLowerInvariant() -cne $Build.binary_sha256) { throw 'Repair did not restore the exact Suite binary' }
} finally {
  if ($Installed) { Invoke-Msi '/x' 'uninstall' }
}
if (Test-Path -LiteralPath (Join-Path $Install 'plervo-suite.exe')) { throw 'Uninstall left Suite installed' }
if (Test-Path -LiteralPath $Shortcut) { throw 'Uninstall left the Suite shortcut' }
if ((Get-Content -LiteralPath $Sentinel) -cne 'Preserve Suite user settings' -or (Get-Content -LiteralPath $Document) -cne 'Preserve user documents') { throw 'Installer changed user data' }
if ((Get-ItemProperty -LiteralPath $ChoicePath -ErrorAction SilentlyContinue).ProgId -cne $ChoiceBefore) { throw 'Suite changed the PDF default handler' }
$Portable = @($Build.files | Where-Object file -Like '*-portable.zip')
if ($Portable.Count -ne 1) { throw 'Expected one portable archive in the candidate manifest' }
$Archive = Join-Path (Split-Path $Msi) $Portable[0].file
if ((Get-FileHash -LiteralPath $Archive -Algorithm SHA256).Hash.ToLowerInvariant() -cne $Portable[0].sha256) { throw 'Portable archive differs from its candidate manifest' }
Expand-Archive -LiteralPath $Archive -DestinationPath (Join-Path $Work 'portable')
$PortableBinary = @(Get-ChildItem -LiteralPath (Join-Path $Work 'portable') -Recurse -File -Filter plervo-suite.exe)
if ($PortableBinary.Count -ne 1 -or (Get-FileHash -LiteralPath $PortableBinary[0].FullName -Algorithm SHA256).Hash.ToLowerInvariant() -cne $Build.binary_sha256) { throw 'Portable executable differs from the tested MSI binary' }
if (-not (Test-Path -LiteralPath (Join-Path $PortableBinary[0].DirectoryName 'portable.txt'))) { throw 'Portable mode marker missing' }
if ((Invoke-Suite -Arguments @('--version') -Executable $PortableBinary[0].FullName -Label 'portable-version').Trim() -cne "Plervo Suite $($Build.version)") { throw 'Portable Suite failed' }
[ordered] @{ passed = $true; source_commit = $Build.source_commit; version = $Build.version; signed = $false; public_qualification = $false;
  checks = @('compiled MSI scope and identity', 'Start Menu registration/removal', 'no PDF shell options or MSI portable mode', 'install and exact binary', 'CLI version/status/input rejection', 'native Suite screenshot', 'repair missing executable', 'uninstall preserves settings/documents/default handler', 'portable ZIP identity and executable') } |
  ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $Evidence 'result.json') -Encoding utf8
