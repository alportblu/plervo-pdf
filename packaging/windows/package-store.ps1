<# Build a Store-managed x64 MSIX. Microsoft signs the submission after certification. #>
param(
  [Parameter(Mandatory)] [ValidatePattern('^[A-Za-z0-9][A-Za-z0-9.-]{2,49}$')] [string] $IdentityName,
  [Parameter(Mandatory)] [ValidateLength(3, 8192)] [string] $Publisher,
  [Parameter(Mandatory)] [ValidateLength(1, 256)] [string] $PublisherDisplayName,
  [string] $Version = '1.0.7.0',
  [switch] $ValidationOnly,
  [switch] $ManifestOnly
)
$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$ParsedVersion = [version] $Version
if ($Version -notmatch '^\d+\.\d+\.\d+\.0$' -or $ParsedVersion.Major -lt 1 -or
    $ParsedVersion.Major -gt 65535 -or $ParsedVersion.Minor -gt 65535 -or $ParsedVersion.Build -gt 65535) {
  throw 'Store versions require four fields: major 1..65535, minor/build 0..65535, revision 0.'
}
[void] [Security.Cryptography.X509Certificates.X500DistinguishedName]::new($Publisher)
if ($ManifestOnly -and -not $ValidationOnly) {
  throw 'Manifest-only preflight requires -ValidationOnly and cannot produce a submission package.'
}
if (-not $ManifestOnly -and (-not $env:CRAFT_FONTS_DIR -or -not (Test-Path (Join-Path $env:CRAFT_FONTS_DIR 'fonts')))) {
  throw 'Store builds require the pinned CRAFT_FONTS_DIR input.'
}
if ($Publisher -match 'OID\.2\.25\.311729368913984317654407730594956997722') {
  throw 'Supply the real publisher; the test-only OID is added only with -ValidationOnly.'
}

function Invoke-Native([string] $Label, [scriptblock] $Action) {
  & $Action
  if ($LASTEXITCODE -ne 0) { throw "$Label failed with exit code $LASTEXITCODE" }
}

$MakeAppx = Get-ChildItem -Path "${env:ProgramFiles(x86)}\Windows Kits\10\bin" -Filter makeappx.exe -Recurse |
  Where-Object FullName -Match '\\x64\\makeappx\.exe$' |
  Sort-Object FullName -Descending | Select-Object -First 1
if (-not $MakeAppx) { throw 'Windows SDK makeappx.exe (x64) is required.' }
$Target = 'x86_64-pc-windows-msvc'
$TargetDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $Root 'target' }
$Dist = Join-Path $Root 'dist\store'
$StageBase = if ($ManifestOnly) { [IO.Path]::GetTempPath() } else { $TargetDir }
$Stage = Join-Path $StageBase ('store-stage-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $Stage | Out-Null
$Probe = $null
$env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS = '-C target-feature=+crt-static'
$env:PDFCRAFT_REQUIRE_WINRES = '1'
$env:PDFCRAFT_BUILD_LABEL = 'Microsoft Store preview'
$env:PDFCRAFT_BUILD_SHA = (git -C $Root rev-parse HEAD)
$env:PDFCRAFT_BUILD_DATE = (Get-Date).ToUniversalTime().ToString('yyyy-MM-dd')
Push-Location $Root
try {
  Copy-Item -Recurse (Join-Path $Root 'assets\app-icon\store') (Join-Path $Stage 'Assets')
  $Template = Get-Content -Raw (Join-Path $PSScriptRoot 'store\AppxManifest.xml.in')
  $StorePublisher = $Publisher
  $Suffix = ''
  if ($ValidationOnly) {
    # This namespace permits testing without installing a certificate or weakening OS policy.
    $Publisher += ', OID.2.25.311729368913984317654407730594956997722=1'
    $Suffix = '-validation-only'
  }
  $Values = @{
    '@IDENTITY_NAME@' = $IdentityName
    '@PUBLISHER@' = $Publisher
    '@PUBLISHER_DISPLAY_NAME@' = $PublisherDisplayName
    '@VERSION@' = $Version
  }
  foreach ($Key in $Values.Keys) { $Template = $Template.Replace($Key, [Security.SecurityElement]::Escape($Values[$Key])) }
  if ($Template -match '@[A-Z_]+@') { throw 'Manifest has an unresolved placeholder.' }
  $Template | Set-Content -LiteralPath (Join-Path $Stage 'AppxManifest.xml') -Encoding utf8
  if ($ManifestOnly) {
    # Inert files satisfy manifest references; this checks SDK semantics, never app behavior.
    # The temporary probe is neither installed nor retained as an artifact.
    foreach ($Name in 'plervo-pdf.exe', 'plervo-pdf-cli.exe') {
      [IO.File]::WriteAllBytes((Join-Path $Stage $Name), [byte[]] @())
    }
    $Probe = "$Stage.msix"
    Invoke-Native 'Manifest-only SDK preflight' { & $MakeAppx.FullName pack /d $Stage /p $Probe /o }
    Write-Host 'Manifest preflight passed; real binaries and native lifecycle still require validation.'
    return
  }
  New-Item -ItemType Directory -Force -Path $Dist | Out-Null
  Invoke-Native 'Third-party distribution notices' { cargo xtask licenses }
  Invoke-Native 'Store binaries' {
    cargo build --release --locked -p plervo-pdf -p plervo-pdf-cli --features plervo-pdf/microsoft-store --target $Target
  }
  $Bin = Join-Path $TargetDir "$Target\release"
  Copy-Item (Join-Path $Bin 'plervo-pdf.exe'), (Join-Path $Bin 'plervo-pdf-cli.exe') $Stage
  foreach ($Name in 'LICENSE-MIT', 'LICENSE-APACHE', 'NOTICE', 'ATTRIBUTION.md', 'THIRD_PARTY_NOTICES.txt') {
    Copy-Item (Join-Path $Root $Name) $Stage
  }
  Copy-Item (Join-Path $Root 'assets\app-icon\LICENSE-document.txt') $Stage
  Copy-Item (Join-Path $Root 'assets\icons\LICENSE-lucide.txt') $Stage
  Copy-Item (Join-Path $Root 'assets\fonts\OFL-*.txt') $Stage
  Get-ChildItem -Path (Join-Path $env:CRAFT_FONTS_DIR 'fonts') -Directory | ForEach-Object {
    $Licence = Join-Path $_.FullName 'OFL.txt'
    if (Test-Path -LiteralPath $Licence) { Copy-Item $Licence (Join-Path $Stage "OFL-$($_.Name).txt") }
  }
  $Models = Join-Path $Stage 'models'
  Invoke-Native 'Verified OCR models' { cargo xtask models $Models }
  if (-not (Test-Path (Join-Path $Models 'ATTRIBUTION.txt'))) { throw 'OCR model attribution missing.' }
  $Package = Join-Path $Dist "plervo-pdf-$Version-windows-x64$Suffix.msix"
  Invoke-Native 'MSIX packaging and SDK validation' { & $MakeAppx.FullName pack /d $Stage /p $Package /o }
  $Files = @(Get-ChildItem -LiteralPath $Stage -Recurse -File | ForEach-Object {
    @{ path = [IO.Path]::GetRelativePath($Stage, $_.FullName); sha256 = (Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant() }
  })
  @{
    source_commit = $env:PDFCRAFT_BUILD_SHA
    identity_name = $IdentityName
    publisher = $StorePublisher
    publisher_display_name = $PublisherDisplayName
    package_version = $Version
    validation_only = [bool] $ValidationOnly
    microsoft_store_certified = $false
    package = [IO.Path]::GetFileName($Package)
    sha256 = (Get-FileHash $Package -Algorithm SHA256).Hash.ToLowerInvariant()
    files = $Files
  } | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath "$Package.json" -Encoding utf8
  Get-Item -LiteralPath $Package | Select-Object Name, Length
} finally {
  Pop-Location
  if ($Probe -and (Test-Path -LiteralPath $Probe)) { Remove-Item -LiteralPath $Probe -Force }
  Remove-Item -LiteralPath $Stage -Recurse -Force
}
