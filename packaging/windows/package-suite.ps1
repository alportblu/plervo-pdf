<# Internal Windows x64 Suite packages. No signing secrets, public upload or trust-store change. #>
param([switch] $SkipBuild)
$ErrorActionPreference = 'Stop'
if (-not $IsWindows) { throw 'Build the Windows Suite on Windows with PowerShell 7.' }
$Root = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$TargetDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $Root 'target' }
$Dist = Join-Path $Root 'dist/windows-suite-internal'
$inPackage = $false
$Version = $null
foreach ($line in Get-Content (Join-Path $Root 'Cargo.toml')) {
  if ($line -match '^\s*\[') { $inPackage = $line.Trim() -eq '[workspace.package]'; continue }
  if ($inPackage -and $line -match '^version\s*=\s*"([^"]+)"') { $Version = $Matches[1]; break }
}
if ($Version -notmatch '^\d+\.\d+\.\d+(?:-[A-Za-z0-9.]+)?$') { throw 'Invalid workspace version' }
if (-not $SkipBuild) {
  $env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS = '-C target-feature=+crt-static'
  cargo build --release --locked --target x86_64-pc-windows-msvc -p plervo-suite
  if ($LASTEXITCODE -ne 0) { throw 'Suite compilation failed' }
}
$Binary = Join-Path $TargetDir 'x86_64-pc-windows-msvc/release/plervo-suite.exe'
$length = (Get-Item -LiteralPath $Binary).Length
if ($length -lt 256 -or $length -gt 512MB) { throw 'Invalid Suite executable size' }
$bytes = [IO.File]::ReadAllBytes($Binary)
if ($bytes[0] -ne 0x4D -or $bytes[1] -ne 0x5A) { throw 'Invalid Suite PE file' }
$offset = [BitConverter]::ToInt32($bytes, 0x3C)
if ($offset -lt 64 -or $offset -gt $bytes.Length - 96 -or [BitConverter]::ToUInt32($bytes, $offset) -ne 0x4550) { throw 'Invalid Suite PE header' }
if ([BitConverter]::ToUInt16($bytes, $offset + 4) -ne 0x8664 -or [BitConverter]::ToUInt16($bytes, $offset + 0x5C) -ne 2) { throw 'Expected a Windows x64 GUI executable' }
$info = (Get-Item -LiteralPath $Binary).VersionInfo
if ($info.ProductName -cne 'Plervo Suite' -or $info.ProductVersion -cne $Version) { throw 'Missing or incorrect Suite version resources' }
$Source = (git -C $Root rev-parse HEAD).Trim()
if ($LASTEXITCODE -ne 0 -or $Source -notmatch '^[a-f0-9]{40}$') { throw 'Missing source commit' }
$Work = Join-Path $TargetDir ('suite-windows-package-' + [Guid]::NewGuid())
$Name = "plervo-suite-$Version-internal-windows-x64"
$Stage = Join-Path $Work $Name
New-Item -ItemType Directory -Path (Join-Path $Stage 'docs') -Force | Out-Null
New-Item -ItemType Directory -Path $Dist -Force | Out-Null
try {
  Copy-Item -LiteralPath $Binary -Destination (Join-Path $Stage 'plervo-suite.exe')
  foreach ($file in 'README.md', 'LICENSE-MIT', 'LICENSE-APACHE', 'NOTICE', 'ATTRIBUTION.md') {
    Copy-Item -LiteralPath (Join-Path $Root $file) -Destination (Join-Path $Stage 'docs')
  }
  Copy-Item (Join-Path $Root 'assets/fonts/OFL-*.txt') (Join-Path $Stage 'docs')
  Copy-Item (Join-Path $Root 'assets/icons/LICENSE-lucide.txt') (Join-Path $Stage 'docs')
  'Internal, unsigned candidate. Windows app downloads remain disabled pending trusted-publisher integration and acceptance. The installer UI is currently English; the Suite follows its own system-language setting.' |
    Set-Content -LiteralPath (Join-Path $Stage 'docs/INTERNAL-CANDIDATE.txt') -Encoding utf8
  $Msi = Join-Path $Dist "$Name.msi"
  $Zip = Join-Path $Dist "$Name-portable.zip"
  if ((Test-Path -LiteralPath $Msi) -or (Test-Path -LiteralPath $Zip)) { throw 'Refusing to overwrite an existing candidate' }
  wix build (Join-Path $PSScriptRoot 'plervo-suite.wxs') (Join-Path $PSScriptRoot 'installer-ui.wxs') -arch x64 `
    -d "Version=$(($Version -split '-')[0])" -d "BinDir=$Stage" -d "IconPath=$(Join-Path $Root 'assets/products/suite/icon.ico')" `
    -d 'IncludeShellOptions=0' -o $Msi
  if ($LASTEXITCODE -ne 0) { throw 'Suite MSI build failed' }
  Remove-Item -LiteralPath ([IO.Path]::ChangeExtension($Msi, '.wixpdb')) -ErrorAction SilentlyContinue
  # The MSI never gets portable.txt; its preferences stay outside Program Files and survive removal.
  'Plervo Suite portable mode' | Set-Content -LiteralPath (Join-Path $Stage 'portable.txt') -Encoding utf8
  Compress-Archive -LiteralPath $Stage -DestinationPath $Zip
  $Files = @($Msi, $Zip) | ForEach-Object {
    [ordered] @{ file = [IO.Path]::GetFileName($_); bytes = (Get-Item -LiteralPath $_).Length; sha256 = (Get-FileHash -LiteralPath $_ -Algorithm SHA256).Hash.ToLowerInvariant() }
  }
  [ordered] @{ product = 'Plervo Suite'; version = $Version; source_commit = $Source; platform = 'windows-x64'; public_qualification = $false; signed = $false;
    binary_sha256 = (Get-FileHash -LiteralPath $Binary -Algorithm SHA256).Hash.ToLowerInvariant(); files = $Files } |
    ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $Dist 'PLERVO-SUITE-BUILD.json') -Encoding utf8
  $Files | ForEach-Object { "$($_.sha256)  $($_.file)" } | Set-Content -LiteralPath (Join-Path $Dist 'SHA256SUMS.txt') -Encoding utf8
  Get-Item -LiteralPath $Msi, $Zip | Format-Table Name, Length
} finally {
  Remove-Item -LiteralPath $Work -Recurse -Force
}
