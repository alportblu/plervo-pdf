<# Install only our unsigned test namespace on a disposable Windows CI runner. #>
param([Parameter(Mandatory)] [ValidateSet('video', 'cad')] [string] $Product)
$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_OS -ne 'Windows') { throw 'Disposable Windows GitHub runner required.' }
$Root = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..')).Path
$Packages = @(Get-ChildItem (Join-Path $Root "dist\store\$Product\*-validation-only.msix"))
if ($Packages.Count -ne 1) { throw 'Expected exactly one validation-only package.' }
$Package = $Packages[0].FullName
$Metadata = Get-Content -Raw "$Package.json" | ConvertFrom-Json
$Config = (Get-Content -Raw (Join-Path $PSScriptRoot 'apps.json') | ConvertFrom-Json).$Product
if (-not $Metadata.validation_only -or $Metadata.product -cne $Product -or
    $Metadata.identity_name -cne "Plervo.$Product.StoreValidation" -or
    (Get-FileHash $Package).Hash.ToLowerInvariant() -cne $Metadata.sha256) { throw 'Intact test package required.' }
if (Get-AppxPackage -Name $Metadata.identity_name) { throw 'Refusing to replace an existing package.' }
$Evidence = (New-Item -ItemType Directory -Force (Join-Path $Root "dist\validation\$Product\installed")).FullName
$Work = (New-Item -ItemType Directory (Join-Path $env:RUNNER_TEMP ('plervo-store-test-' + [Guid]::NewGuid().ToString('N')))).FullName
$ChoicePath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\$($Config.extension)\UserChoice"
$BeforeChoice = (Get-ItemProperty -LiteralPath $ChoicePath -ErrorAction SilentlyContinue).ProgId
$Aliases = Join-Path $env:LOCALAPPDATA 'Microsoft\WindowsApps'
$Cli = Join-Path $Aliases "plervo-$Product-cli.exe"
$Executable = Join-Path $Aliases "plervo-$Product.exe"
$Document = Join-Path $Work "acceptance$($Config.extension)"
$DocumentHash = $null
function Invoke-Cli([string[]] $Arguments) {
  $Text = & $script:Cli @Arguments
  if ($LASTEXITCODE -ne 0) { throw "Installed CLI failed: $($Arguments[0]), exit $LASTEXITCODE" }
  return $Text
}
try {
  Add-AppxPackage -Path $Package -AllowUnsigned
  $Installed = Get-AppxPackage -Name $Metadata.identity_name
  if (-not $Installed) { throw 'Package was not registered.' }
  $Manifest = Get-AppxPackageManifest -Package $Installed.PackageFullName
  if ($Manifest.Package.Identity.Publisher -notmatch 'OID\.2\.25\.311729368913984317654407730594956997722=1') { throw 'Not the unsigned testing namespace.' }
  foreach ($File in $Metadata.files) {
    if ((Get-FileHash (Join-Path $Installed.InstallLocation $File.path)).Hash.ToLowerInvariant() -cne $File.sha256) { throw "Installed file differs: $($File.path)" }
  }
  if (Test-Path (Join-Path $Installed.InstallLocation 'portable.txt')) { throw 'MSIX must use per-user writable storage.' }
  if (-not (Test-Path $Cli) -or -not (Test-Path $Executable)) { throw 'Execution aliases missing.' }
  $VersionText = Invoke-Cli @('--version')
  if ($VersionText -notmatch [regex]::Escape($Metadata.release_version)) { throw 'Installed product version differs.' }
  $VersionText | Set-Content (Join-Path $Evidence 'version.txt')
  $Render = Join-Path $Evidence 'installed-render.png'
  if ($Product -eq 'video') {
    $Data = Join-Path $Work 'video-data'
    [void] (Invoke-Cli @('--data-dir', $Data, '--demo', 'exec', 'file.saveAs', "path=$Document"))
    [void] (Invoke-Cli @('--data-dir', $Data, '--project', $Document, 'render', '--seconds', '1', '--scale', '0.25', '--out', $Render))
  } else {
    $env:PLERVO_CAD_CONFIG_DIR = Join-Path $Work 'cad-config'
    [void] (Invoke-Cli @('run', '--metric', '--cmd', 'line {"points":[[0,0],[30,20],[45,0]]}', '--save', $Document, '--export', $Render))
    $Info = Invoke-Cli @('info', $Document) | ConvertFrom-Json
    if ($Info.entityCount -ne 2) { throw 'Installed CAD did not save two line entities.' }
  }
  python (Join-Path $PSScriptRoot 'verify-render.py') $Render
  if ($LASTEXITCODE -ne 0) { throw 'Installed app output is invalid.' }
  $DocumentHash = (Get-FileHash $Document).Hash
  foreach ($Language in 'en', 'pt-br', 'es') {
    if ($Product -eq 'video') {
      $Data = Join-Path $Work "video-$Language"
      New-Item -ItemType Directory -Path $Data | Out-Null
      $Preferences = @{ version = 2; general = @{ interfaceLanguage = $Language } } | ConvertTo-Json -Depth 4
      [IO.File]::WriteAllText((Join-Path $Data 'preferences.json'), $Preferences, [Text.UTF8Encoding]::new($false))
      $Arguments = @('--data-dir', "`"$Data`"", '--no-recover', "`"$Document`"")
    } else {
      $env:CADCRAFT_LOCALE = $Language
      $env:PLERVO_CAD_CONFIG_DIR = Join-Path $Work "cad-$Language"
      $Arguments = @("`"$Document`"")
    }
    & (Join-Path $PSScriptRoot 'capture.ps1') -Executable $Executable -Arguments $Arguments -Folder (Join-Path $Evidence $Language) -ExpectedTitle $Config.name
  }
  # A real higher-version MSIX update must preserve per-user data and the document.
  $AppData = Join-Path $env:LOCALAPPDATA "Packages\$($Installed.PackageFamilyName)\LocalState"
  New-Item -ItemType Directory -Force -Path $AppData | Out-Null
  $Preference = Join-Path $AppData 'acceptance-preference.txt'
  Set-Content $Preference 'preserve data across Store updates'
  $MakeAppx = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin" -Recurse -Filter makeappx.exe |
    Where-Object FullName -Match '\\x64\\makeappx\.exe$' | Sort-Object FullName -Descending | Select-Object -First 1
  $Stage = Join-Path $Work 'upgrade'
  & $MakeAppx.FullName unpack /p $Package /d $Stage /o
  if ($LASTEXITCODE -ne 0) { throw 'Cannot unpack test upgrade.' }
  $ManifestPath = Join-Path $Stage 'AppxManifest.xml'
  [xml] $Xml = Get-Content -Raw $ManifestPath
  $Old = [version] $Xml.Package.Identity.Version
  $Xml.Package.Identity.Version = "$($Old.Major).$($Old.Minor).$($Old.Build + 1).0"
  $Xml.Save($ManifestPath)
  $Upgrade = Join-Path $Work 'upgrade.msix'
  & $MakeAppx.FullName pack /d $Stage /p $Upgrade /o
  if ($LASTEXITCODE -ne 0) { throw 'Cannot pack test upgrade.' }
  Add-AppxPackage -Path $Upgrade -AllowUnsigned
  $Installed = Get-AppxPackage -Name $Metadata.identity_name
  if ($Installed.Version.ToString() -cne $Xml.Package.Identity.Version -or
      (Get-Content $Preference) -cne 'preserve data across Store updates') { throw 'Upgrade lost app data or did not apply.' }
  [void] (Invoke-Cli @('--version'))
} finally {
  $Registered = Get-AppxPackage -Name $Metadata.identity_name
  if ($Registered) { Remove-AppxPackage -Package $Registered.PackageFullName }
}
if (Get-AppxPackage -Name $Metadata.identity_name) { throw 'Package remained after uninstall.' }
if ((Test-Path $Cli) -or (Test-Path $Executable)) { throw 'Aliases remained after uninstall.' }
if (-not $DocumentHash -or (Get-FileHash $Document).Hash -cne $DocumentHash) { throw 'User document was changed by the lifecycle.' }
if ((Get-ItemProperty -LiteralPath $ChoicePath -ErrorAction SilentlyContinue).ProgId -cne $BeforeChoice) { throw 'Default file handler changed.' }
@{ status = 'passed'; product = $Product; package_sha256 = $Metadata.sha256; source_commit = $Metadata.source_commit; checks = @('installed payload hashes and source notices', 'installed CLI save and render', 'native GUI EN/PT/ES with normal exit', 'no default TCP listener', 'MSIX upgrade preserves app data', 'uninstall preserves document and default handler'); limitations = 'Screenshots require visual review; not all features/hardware or Store delivery are tested.' } |
  ConvertTo-Json -Depth 5 | Set-Content (Join-Path $Evidence 'result.json') -Encoding utf8
