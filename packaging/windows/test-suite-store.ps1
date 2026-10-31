<# Exercise only an unsigned test-namespace MSIX on a disposable Windows runner. #>
param([Parameter(Mandatory)] [string] $Path, [Parameter(Mandatory)] [string] $UpgradePath)
$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_OS -ne 'Windows') { throw 'Use a disposable GitHub Windows runner.' }
$Package = (Resolve-Path -LiteralPath $Path).Path
$Metadata = Get-Content -Raw "$Package.json" | ConvertFrom-Json
$OriginalMetadata = $Metadata
$Upgrade = (Resolve-Path -LiteralPath $UpgradePath).Path
$UpgradeMetadata = Get-Content -Raw "$Upgrade.json" | ConvertFrom-Json
if (-not $Metadata.validation_only -or (Get-FileHash $Package).Hash.ToLowerInvariant() -cne $Metadata.sha256) { throw 'Invalid test package.' }
if (-not $UpgradeMetadata.validation_only -or (Get-FileHash $Upgrade).Hash.ToLowerInvariant() -cne $UpgradeMetadata.sha256 -or
    $UpgradeMetadata.identity_name -cne $Metadata.identity_name -or $UpgradeMetadata.publisher -cne $Metadata.publisher -or
    [version]$UpgradeMetadata.package_version -le [version]$Metadata.package_version -or $UpgradeMetadata.source_commit -cne $Metadata.source_commit) { throw 'Invalid upgrade package.' }
if (Get-AppxPackage -Name $Metadata.identity_name) { throw 'Refusing to replace an existing package.' }
$Evidence = Join-Path $PWD 'dist\validation\suite-store'
$Work = Join-Path $env:RUNNER_TEMP ('suite-store-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Force -Path $Evidence, $Work | Out-Null
$Document = Join-Path $Work 'preserve-document.txt'
'Synthetic user document' | Set-Content $Document
$DocumentHash = (Get-FileHash $Document).Hash
$Choice = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.pdf\UserChoice'
$BeforeChoice = (Get-ItemProperty $Choice -ErrorAction SilentlyContinue).ProgId
$Installed = $null
$Alias = Join-Path $env:LOCALAPPDATA 'Microsoft\WindowsApps\plervo-suite.exe'
function Invoke-Suite([string[]] $Arguments, [string] $Label, [int] $ExpectedExit = 0) {
  $Start = [Diagnostics.ProcessStartInfo]::new($Alias)
  $Start.UseShellExecute = $false
  $Start.RedirectStandardOutput = $true
  $Start.RedirectStandardError = $true
  # All arguments are fixed command names/products or generated file paths. Never a shell.
  foreach ($Argument in $Arguments) { if ($Argument.Contains('"') -or $Argument.EndsWith('\') -or $Argument.Contains("`n")) { throw 'Unsupported acceptance argument.' } }
  $Start.Arguments = ($Arguments | ForEach-Object { '"' + $_ + '"' }) -join ' ' 
  $Process = [Diagnostics.Process]::Start($Start)
  try {
    $Output = $Process.StandardOutput.ReadToEndAsync()
    $ErrorText = $Process.StandardError.ReadToEndAsync()
    if (-not $Process.WaitForExit(60000)) { $Process.Kill(); throw "Suite timed out: $Label" }
    $Text = $Output.GetAwaiter().GetResult()
    $Errors = $ErrorText.GetAwaiter().GetResult()
    $Text | Set-Content (Join-Path $Evidence "$Label-stdout.txt")
    $Errors | Set-Content (Join-Path $Evidence "$Label-stderr.txt")
    if ($Process.ExitCode -ne $ExpectedExit) { throw "Suite $Label exit $($Process.ExitCode): $Errors" }
    return $Text
  } finally { $Process.Dispose() }
}
function Verify-Payload {
  foreach ($File in $Metadata.files) {
    if ((Get-FileHash (Join-Path $Installed.InstallLocation $File.path)).Hash.ToLowerInvariant() -cne $File.sha256) { throw "Installed payload differs: $($File.path)" }
  }
}
try {
  Add-AppxPackage -Path $Package -AllowUnsigned
  $Installed = Get-AppxPackage -Name $Metadata.identity_name
  if (-not $Installed) { throw 'Suite did not register.' }
  $Manifest = Get-AppxPackageManifest -Package $Installed.PackageFullName
  if ($Manifest.Package.Identity.Publisher -notmatch 'OID\.2\.25\.311729368913984317654407730594956997722=1') { throw 'Wrong test namespace.' }
  if (@($Manifest.Package.Applications.Application).Count -ne 1 -or $Manifest.Package.Applications.Application.Id -cne 'PlervoSuite') { throw 'Unexpected app registrations.' }
  if ($Manifest.OuterXml -match 'windows.fileTypeAssociation|windows.startupTask|windows.service') { throw 'Unexpected system integration.' }
  Verify-Payload
  if (-not (Test-Path $Alias)) { throw 'Suite execution alias was not registered.' }
  $Status = (Invoke-Suite @('--status') 'inventory').Trim()
  if ($Status -cne '{}') { throw 'Runner unexpectedly has a trusted Store Plervo app.' }
  $Links = [ordered]@{}
  foreach ($Entry in @(@('pdf','9PBWKBZRBJNR'), @('cad','9P5CZZSKKG1Q'), @('video','9NRKLM46BCDG'))) {
    $Link = (Invoke-Suite @('--store-link', $Entry[0]) ($Entry[0] + '-store-link')).Trim()
    if ($Link -cne ('ms-windows-store://pdp/?ProductId=' + $Entry[1])) { throw "Wrong acquisition link: $($Entry[0])." }
    $Links[$Entry[0]] = $Link
  }
  [void] (Invoke-Suite @('--install', 'photo') 'unpublished-product' 1)
  [void] (Invoke-Suite @('--install', '../pdf') 'invalid-product' 1)
  [void] (Invoke-Suite @('--uninstall', 'pdf') 'external-removal-refused' 1)
  [void] (Invoke-Suite @('--open', 'pdf') 'absent-app-refused' 1)
  foreach ($Capture in @(@('en','apps'), @('en','catalog'), @('pt','settings'), @('es','catalog'))) {
    $env:PLERVO_SUITE_PROBE_LANGUAGE = $Capture[0]
    $env:PLERVO_SUITE_PROBE_VIEW = $Capture[1]
    $Name = "$($Capture[0])-$($Capture[1])"
    $Png = Join-Path $Evidence "$Name.png"
    [void] (Invoke-Suite @('--native-smoke-test', $Png) $Name)
    if (-not (Test-Path $Png) -or (Get-Item $Png).Length -lt 1000) { throw "Missing native screenshot: $Name" }
  }
  $Settings = Join-Path $env:LOCALAPPDATA "Packages\$($Installed.PackageFamilyName)\LocalState\preserve-setting.txt"
  New-Item -ItemType Directory -Force -Path (Split-Path $Settings) | Out-Null
  'Synthetic Suite preference' | Set-Content $Settings
  $SettingsHash = (Get-FileHash $Settings).Hash
  Add-AppxPackage -Path $Package -AllowUnsigned -ForceUpdateFromAnyVersion
  if ((Get-FileHash $Settings).Hash -cne $SettingsHash) { throw 'Reinstallation changed app data.' }
  Verify-Payload
  [void] (Invoke-Suite @('--status') 'inventory-after-reinstall')
  Add-AppxPackage -Path $Upgrade -AllowUnsigned
  $Installed = Get-AppxPackage -Name $Metadata.identity_name
  if ($Installed.Version.ToString() -cne $UpgradeMetadata.package_version) { throw 'MSIX upgrade did not change the registered version.' }
  if ((Get-FileHash $Settings).Hash -cne $SettingsHash) { throw 'Upgrade changed app data.' }
  $Metadata = $UpgradeMetadata
  Verify-Payload
  [void] (Invoke-Suite @('--status') 'inventory-after-upgrade')
} finally {
  Remove-Item Env:PLERVO_SUITE_PROBE_LANGUAGE, Env:PLERVO_SUITE_PROBE_VIEW -ErrorAction SilentlyContinue
  if ($Installed) { Remove-AppxPackage -Package $Installed.PackageFullName }
}
if (Get-AppxPackage -Name $Metadata.identity_name) { throw 'Suite remained registered.' }
if ((Get-FileHash $Document).Hash -cne $DocumentHash) { throw 'User document changed.' }
if ((Get-ItemProperty $Choice -ErrorAction SilentlyContinue).ProgId -cne $BeforeChoice) { throw 'Default PDF handler changed.' }
@{ passed = $true; source_commit = $OriginalMetadata.source_commit; package_sha256 = $OriginalMetadata.sha256; tested_identity = $Metadata.identity_name;
  upgrade_sha256 = $UpgradeMetadata.sha256; install = $true; reinstall = $true; upgrade = $true; remove = $true; payload_hashes = $true; local_inventory = $true; native_ui_languages = @('en','pt','es');
  store_acquisition_link = $Links['pdf']; store_acquisition_links = $Links;
  unpublished_product_refused = 'photo'; consumer_store_acquisition = 'not exercised on Windows Server runner';
  trusted_store_app_activation = 'no Store-signed Plervo app installed on runner'; documents_preserved = $true;
  settings_preserved_on_reinstall = $true; default_pdf_handler_preserved = $true } |
  ConvertTo-Json -Depth 4 | Set-Content (Join-Path $Evidence 'result.json')
