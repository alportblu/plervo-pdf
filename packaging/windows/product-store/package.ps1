<# Build Video/CAD MSIX from reviewed, pinned source. No certificate or trust changes. #>
param(
  [Parameter(Mandatory)] [ValidateSet('video', 'cad')] [string] $Product,
  [string] $Source = 'source',
  [switch] $ValidationOnly,
  [switch] $ManifestOnly
)
$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..')).Path
$Config = (Get-Content -Raw (Join-Path $PSScriptRoot 'apps.json') | ConvertFrom-Json).$Product
$Catalog = Get-Content -Raw (Join-Path $Root 'products\sources.json') | ConvertFrom-Json
$Definition = @($Catalog | Where-Object product -eq $Product)[0]
if ($ManifestOnly -and -not $ValidationOnly) { throw 'Preflight must use the validation-only namespace.' }
if ($Config.identity_name -notmatch '^[A-Za-z0-9][A-Za-z0-9.-]{2,49}$' -or
    $Config.package_version -notmatch '^[1-9]\d*\.\d+\.\d+\.0$') { throw 'Invalid Store identity/version.' }
$Version = [version] $Config.package_version
if ($Version.Major -gt 65535 -or $Version.Minor -gt 65535 -or $Version.Build -gt 65535) { throw 'Store version out of range.' }
[void] [Security.Cryptography.X509Certificates.X500DistinguishedName]::new($Config.publisher)
$MakeAppx = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin" -Recurse -Filter makeappx.exe |
  Where-Object FullName -Match '\\x64\\makeappx\.exe$' | Sort-Object FullName -Descending | Select-Object -First 1
if (-not $MakeAppx) { throw 'Windows SDK makeappx.exe is required.' }
$Stage = Join-Path ([IO.Path]::GetTempPath()) ('plervo-msix-stage-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $Stage | Out-Null
$Probe = $null
$SourceRoot = $null
function Invoke-Native([string] $Label, [scriptblock] $Action) {
  & $Action
  if ($LASTEXITCODE -ne 0) { throw "$Label failed: $LASTEXITCODE" }
}
try {
  # Derive the required tile sizes from the already attributed original Plervo icon.
  Add-Type -AssemblyName System.Drawing
  $IconPath = Join-Path $Root "assets\products\$Product\icon-1024.png"
  $Icon = [Drawing.Image]::FromFile($IconPath)
  $Assets = New-Item -ItemType Directory -Path (Join-Path $Stage 'Assets')
  $AssetCredits = @()
  try {
    foreach ($Tile in @(@('Square44x44Logo.png', 44), @('Square150x150Logo.png', 150), @('StoreLogo.png', 50))) {
      $Bitmap = [Drawing.Bitmap]::new($Tile[1], $Tile[1])
      $Graphics = [Drawing.Graphics]::FromImage($Bitmap)
      try {
        $Graphics.InterpolationMode = [Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
        $Graphics.DrawImage($Icon, 0, 0, $Tile[1], $Tile[1])
        $File = Join-Path $Assets.FullName $Tile[0]
        $Bitmap.Save($File, [Drawing.Imaging.ImageFormat]::Png)
        $AssetCredits += [ordered] @{ path = "Assets/$($Tile[0])"; sha256 = (Get-FileHash $File).Hash.ToLowerInvariant(); source = "assets/products/$Product/icon-1024.png"; source_sha256 = (Get-FileHash $IconPath).Hash.ToLowerInvariant(); author = 'Plervo contributors'; license = 'MIT OR Apache-2.0'; license_files = @('Licenses/LICENSE-MIT', 'Licenses/LICENSE-APACHE'); modification = 'Resized to the required Microsoft Store tile dimensions.' }
      } finally { $Graphics.Dispose(); $Bitmap.Dispose() }
    }
  } finally { $Icon.Dispose() }
  $Identity = $Config.identity_name
  $Publisher = $Config.publisher
  $Suffix = ''
  if ($ValidationOnly) {
    $Identity = "Plervo.$Product.StoreValidation"
    $Publisher += ', OID.2.25.311729368913984317654407730594956997722=1'
    $Suffix = '-validation-only'
  }
  $Template = Get-Content -Raw (Join-Path $PSScriptRoot 'AppxManifest.xml.in')
  $Values = @{
    '@IDENTITY_NAME@' = $Identity; '@PUBLISHER@' = $Publisher
    '@PUBLISHER_DISPLAY_NAME@' = $Config.publisher_display_name; '@VERSION@' = $Config.package_version
    '@DISPLAY_NAME@' = $Config.name; '@APP_ID@' = $Config.application_id
    '@DESCRIPTION@' = $Config.description; '@PRODUCT@' = $Product
    '@FILE_EXTENSION@' = $Config.extension; '@FILE_TYPE@' = $Config.file_type
  }
  foreach ($Key in $Values.Keys) { $Template = $Template.Replace($Key, [Security.SecurityElement]::Escape($Values[$Key])) }
  if ($Template -match '@[A-Z_]+@') { throw 'Unresolved manifest placeholder.' }
  $Template | Set-Content -LiteralPath (Join-Path $Stage 'AppxManifest.xml') -Encoding utf8
  if ($ManifestOnly) {
    foreach ($Name in "plervo-$Product.exe", "plervo-$Product-cli.exe") { [IO.File]::WriteAllBytes((Join-Path $Stage $Name), [byte[]] @()) }
    $Probe = "$Stage.msix"
    Invoke-Native 'SDK-only manifest preflight' { & $MakeAppx.FullName pack /d $Stage /p $Probe /o }
    return
  }
  $SourceRoot = (Resolve-Path -LiteralPath $Source).Path
  $Provenance = Get-Content -Raw (Join-Path $SourceRoot 'PLERVO-STORE-BUILD.json') | ConvertFrom-Json
  if ($Provenance.product -cne $Product -or $Provenance.base_patch_sha256 -cne $Definition.patch_sha256 -or
      $Provenance.upstream_commit -cne $Definition.commit -or
      (Get-FileHash (Join-Path $SourceRoot 'Cargo.lock')).Hash.ToLowerInvariant() -cne $Provenance.lock_sha256) { throw 'Unreviewed prepared source.' }
  if (-not $env:CRAFT_FONTS_DIR) { throw 'Pinned fonts are required.' }
  $FontHead = git -C $env:CRAFT_FONTS_DIR rev-parse HEAD
  if ($LASTEXITCODE -ne 0 -or $FontHead -cne $Definition.fonts_commit) { throw 'Font source differs from the catalog.' }
  $SourceCommit = git -C $Root rev-parse HEAD
  if ($LASTEXITCODE -ne 0) { throw 'Cannot identify the build source.' }
  $Prefix = $Definition.app.ToUpperInvariant()
  [Environment]::SetEnvironmentVariable("${Prefix}_REQUIRE_WINRES", '1')
  [Environment]::SetEnvironmentVariable("${Prefix}_BUILD_SHA", $SourceCommit)
  [Environment]::SetEnvironmentVariable("${Prefix}_BUILD_DATE", (Get-Date).ToUniversalTime().ToString('yyyy-MM-dd'))
  $env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS = '-C target-feature=+crt-static'
  Push-Location $SourceRoot
  try { Invoke-Native 'Public desktop and CLI (default features only)' { cargo build --release --locked -p "plervo-$Product" -p "plervo-$Product-cli" --target x86_64-pc-windows-msvc } }
  finally { Pop-Location }
  $Bin = Join-Path $SourceRoot 'target\x86_64-pc-windows-msvc\release'
  foreach ($Check in @(@("plervo-$Product.exe", 2), @("plervo-$Product-cli.exe", 3))) {
    $File = Join-Path $Bin $Check[0]
    $Bytes = [IO.File]::ReadAllBytes($File)
    $Pe = [BitConverter]::ToInt32($Bytes, 0x3C)
    if ([BitConverter]::ToUInt16($Bytes, $Pe + 4) -ne 0x8664 -or [BitConverter]::ToUInt16($Bytes, $Pe + 0x5C) -ne $Check[1]) { throw 'Wrong executable architecture or subsystem.' }
    Copy-Item -LiteralPath $File -Destination $Stage
  }
  $Legal = New-Item -ItemType Directory -Path (Join-Path $Stage 'Licenses')
  foreach ($Name in 'LICENSE-MIT', 'LICENSE-APACHE', 'NOTICE', 'ATTRIBUTION.md') { Copy-Item (Join-Path $SourceRoot $Name) $Legal.FullName }
  Copy-Item (Join-Path $SourceRoot 'THIRD_PARTY_NOTICES.txt') $Legal.FullName
  Copy-Item (Join-Path $SourceRoot 'packaging\licenses\manifest.json') (Join-Path $Legal.FullName 'DEPENDENCY-LICENSES.json')
  if ((Get-FileHash (Join-Path $Legal.FullName 'THIRD_PARTY_NOTICES.txt')).Hash.ToLowerInvariant() -cne $Definition.notices_sha256 -or
      (Get-FileHash (Join-Path $Legal.FullName 'DEPENDENCY-LICENSES.json')).Hash.ToLowerInvariant() -cne $Definition.notice_manifest_sha256) {
    throw 'Bundled dependency notices differ from the reviewed catalog.'
  }
  foreach ($Folder in 'assets', 'vendor') {
    $Path = Join-Path $SourceRoot $Folder
    if (Test-Path $Path) {
      Get-ChildItem $Path -Recurse -File | Where-Object Name -Match '(?i)licen[cs]e|OFL.*txt|copyright|^notice|^PLERVO-PATCH\.md$' | ForEach-Object {
        $Name = ([IO.Path]::GetRelativePath($SourceRoot, $_.FullName) -replace '[\\/]', '_') + '.txt'
        Copy-Item $_.FullName (Join-Path $Legal.FullName $Name)
      }
    }
  }
  $FontLicenses = @(Get-ChildItem (Join-Path $env:CRAFT_FONTS_DIR 'fonts\*\OFL.txt'))
  if ($FontLicenses.Count -eq 0) { throw 'No font notices found.' }
  foreach ($File in $FontLicenses) { Copy-Item $File.FullName (Join-Path $Legal.FullName "OFL-$($File.Directory.Name).txt") }
  foreach ($Name in 'LICENSE-MIT', 'LICENSE-APACHE') {
    if ((Get-FileHash (Join-Path $Legal.FullName $Name)).Hash.ToLowerInvariant() -cne $Provenance.licenses.$Name) { throw "License text differs: $Name" }
  }
  Copy-Item (Join-Path $SourceRoot 'PLERVO-BUILD.json'), (Join-Path $SourceRoot 'PLERVO-STORE-BUILD.json') $Legal.FullName
  $AssetCredits | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $Legal.FullName 'STORE-ASSETS.json') -Encoding utf8
  $Dist = New-Item -ItemType Directory -Force -Path (Join-Path $Root "dist\store\$Product")
  $Package = Join-Path $Dist.FullName "plervo-$Product-$($Config.package_version)-windows-x64$Suffix.msix"
  Invoke-Native 'Package and validate MSIX' { & $MakeAppx.FullName pack /d $Stage /p $Package /o }
  $Files = @(Get-ChildItem $Stage -Recurse -File | ForEach-Object { @{ path = [IO.Path]::GetRelativePath($Stage, $_.FullName); sha256 = (Get-FileHash $_.FullName).Hash.ToLowerInvariant() } })
  @{ product = $Product; source_commit = $SourceCommit; identity_name = $Identity; publisher = $Config.publisher; package_version = $Config.package_version; release_version = $Definition.release_version; validation_only = [bool] $ValidationOnly; store_certification = 'not submitted'; sha256 = (Get-FileHash $Package).Hash.ToLowerInvariant(); files = $Files; source = $Provenance } |
    ConvertTo-Json -Depth 8 | Set-Content "$Package.json" -Encoding utf8
  Get-Item $Package | Select-Object Name, Length
} finally {
  if ($Probe -and (Test-Path $Probe)) { Remove-Item -LiteralPath $Probe -Force }
  Remove-Item -LiteralPath $Stage -Recurse -Force
}
