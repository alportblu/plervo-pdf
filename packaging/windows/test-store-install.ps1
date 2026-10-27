<# Exercise an unsigned test-namespace MSIX only on a disposable GitHub Windows runner. #>
param([Parameter(Mandatory)] [string] $Path)
$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_OS -ne 'Windows') {
  throw 'MSIX acceptance runs only on disposable GitHub Windows runners.'
}
$Package = (Resolve-Path -LiteralPath $Path).Path
$Metadata = Get-Content -Raw -LiteralPath "$Package.json" | ConvertFrom-Json
if (-not $Metadata.validation_only -or (Get-FileHash $Package -Algorithm SHA256).Hash.ToLowerInvariant() -cne $Metadata.sha256) {
  throw 'Expected an intact validation-only package, never a customer package.'
}
if (Get-AppxPackage -Name $Metadata.identity_name) { throw 'Refusing to replace an existing package.' }
$Evidence = Join-Path $PWD 'dist\validation\store'
$Work = Join-Path $env:RUNNER_TEMP ('plervo-msix-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Force -Path $Evidence, $Work | Out-Null
$Document = Join-Path $Work 'Plervo PDF sample.pdf'
$Control = Join-Path $Work 'control.json'
$ChoicePath = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.pdf\UserChoice'
$BeforeChoice = (Get-ItemProperty -LiteralPath $ChoicePath -ErrorAction SilentlyContinue).ProgId
$env:PDFCRAFT_SYSTEM_FONTS = '0'
$App = $null
$Installed = $null

function Invoke-Cli([string[]] $Arguments) {
  $Text = & $script:Cli @Arguments
  if ($LASTEXITCODE -ne 0) { throw "Installed CLI failed: $($Arguments[0]) (exit $LASTEXITCODE)." }
  return $Text
}
function Invoke-Ui([string[]] $Arguments) {
  return Invoke-Cli (@('ui', '--control', $Control) + $Arguments)
}

try {
  # Microsoft's unsigned-package test namespace scopes this exception to this package.
  # No trusted root, developer-mode toggle or SmartScreen change is made.
  Add-AppxPackage -Path $Package -AllowUnsigned
  $Installed = Get-AppxPackage -Name $Metadata.identity_name
  if (-not $Installed) { throw 'MSIX did not register for the runner user.' }
  $Manifest = Get-AppxPackageManifest -Package $Installed.PackageFullName
  if ($Manifest.Package.Identity.Publisher -notmatch 'OID\.2\.25\.311729368913984317654407730594956997722=1') {
    throw 'The installed package is outside the unsigned testing namespace.'
  }
  $Install = $Installed.InstallLocation
  foreach ($File in $Metadata.files) {
    $Actual = (Get-FileHash -LiteralPath (Join-Path $Install $File.path) -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($Actual -cne $File.sha256) { throw "Installed file differs: $($File.path)" }
  }
  if (Test-Path (Join-Path $Install 'portable.txt')) { throw 'MSIX must not write data beside its executable.' }
  # Activate through registered aliases; WindowsApps payload paths are protected.
  $AliasRoot = Join-Path $env:LOCALAPPDATA 'Microsoft\WindowsApps'
  $Cli = Join-Path $AliasRoot 'plervo-pdf-cli.exe'
  $Executable = Join-Path $AliasRoot 'plervo-pdf.exe'
  if (-not (Test-Path $Cli) -or -not (Test-Path $Executable)) { throw 'MSIX execution aliases were not registered.' }
  Invoke-Cli @('--version') | Set-Content (Join-Path $Evidence 'cli-version.txt')
  $Notices = Get-Content -Raw -LiteralPath (Join-Path $Install 'THIRD_PARTY_NOTICES.txt')
  foreach ($Notice in '2016 Junfeng Liu', '2011 Google Inc.', '2020 Yevhenii Reizner', 'UBUNTU FONT LICENCE', 'Bitstream') {
    if (-not $Notices.Contains($Notice)) { throw "Missing installed third-party notice: $Notice" }
  }
  # A compound painted path holds synthetic digits under a mark and a shape outside it.
  # Clipping this path used to retain the secret while reporting a successful redaction.
  $VectorBody = "0 g 50 100 8 50 re 75 142 32 8 re 99 125 8 25 re 75 121 32 8 re 75 100 8 25 re 75 100 32 8 re 220 40 20 20 re f`n"
  $Objects = @(
    '<< /Type /Catalog /Pages 2 0 R >>',
    '<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
    '<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 200] /Resources << >> /Contents 4 0 R /Annots [5 0 R] >>',
    "<< /Length $($VectorBody.Length) >>`nstream`n${VectorBody}endstream",
    '<< /Type /Annot /Subtype /Redact /Rect [40 90 165 160] /IC [0 0 0] /P 3 0 R >>'
  )
  $Pdf = [Text.StringBuilder]::new("%PDF-1.7`n")
  $Offsets = @()
  for ($i = 0; $i -lt $Objects.Count; $i++) {
    $Offsets += $Pdf.Length
    [void] $Pdf.Append("$($i + 1) 0 obj`n$($Objects[$i])`nendobj`n")
  }
  $Xref = $Pdf.Length
  [void] $Pdf.Append("xref`n0 6`n0000000000 65535 f `n")
  foreach ($Offset in $Offsets) { [void] $Pdf.Append(('{0:D10} 00000 n ' -f $Offset) + "`n") }
  [void] $Pdf.Append("trailer`n<< /Size 6 /Root 1 0 R >>`nstartxref`n$Xref`n%%EOF`n")
  $Vector = Join-Path $Work 'vector.pdf'
  [IO.File]::WriteAllText($Vector, $Pdf.ToString(), [Text.Encoding]::ASCII)
  $VectorHash = (Get-FileHash $Vector -Algorithm SHA256).Hash
  $VectorScript = Join-Path $Work 'vector.json'
  [IO.File]::WriteAllText($VectorScript, '[{"tool":"doc_open","args":{"path":"vector.pdf"}},{"tool":"redact_apply","args":{"doc":1}},{"tool":"doc_save","args":{"doc":1,"path":"unsafe-output.pdf"}}]', [Text.UTF8Encoding]::new($false))
  $VectorError = Join-Path $Evidence 'vector-redaction-error.txt'
  $Probe = Start-Process -FilePath $Cli -ArgumentList @('run', '--root', "`"$Work`"", '--script', "`"$VectorScript`"") -Wait -PassThru -NoNewWindow -RedirectStandardError $VectorError -RedirectStandardOutput (Join-Path $Evidence 'vector-redaction-output.txt')
  if ($Probe.ExitCode -eq 0 -or (Get-Content -Raw $VectorError) -notmatch 'cannot permanently remove') { throw 'Installed CLI accepted recoverable vector redaction or failed for the wrong reason.' }
  if ((Test-Path (Join-Path $Work 'unsafe-output.pdf')) -or (Get-FileHash $Vector -Algorithm SHA256).Hash -cne $VectorHash) { throw 'Refused redaction changed a file.' }
  $TextScript = Join-Path $Work 'text-redaction.json'
  [IO.File]::WriteAllText($TextScript, '[{"tool":"doc_create","args":{"from":"text","text":"Public SECRET42 remains"}},{"tool":"redact_mark","args":{"doc":1,"find":"SECRET42"}},{"tool":"redact_apply","args":{"doc":1}},{"tool":"doc_save","args":{"doc":1,"path":"safe-output.pdf"}}]', [Text.UTF8Encoding]::new($false))
  [void] (Invoke-Cli @('run', '--root', $Work, '--script', $TextScript))
  $RedactedText = (Invoke-Cli @('text', (Join-Path $Work 'safe-output.pdf'))) -join "`n"
  if ($RedactedText.Contains('SECRET42') -or -not $RedactedText.Contains('Public') -or -not $RedactedText.Contains('remains')) { throw 'Installed CLI failed ordinary text redaction.' }
  $Ocr = Invoke-Cli @('run', 'ocr_status') | ConvertFrom-Json
  if (-not $Ocr.available) { throw 'Packaged OCR models were not found.' }
  $Steps = Join-Path $Work 'create.json'
  $CreateScript = @(
    @{ tool = 'doc_create'; args = @{ from = 'text'; text = "PLERVO PDF`n`nA clearer workspace for your documents.`n`nRead, annotate and organize PDF files.`n`nWork locally. Keep control of your documents.`n`nFree and open source." } },
    @{ tool = 'doc_save'; args = @{ doc = 1; path = [IO.Path]::GetFileName($Document) } }
  ) | ConvertTo-Json -Depth 5
  # Windows PowerShell's Set-Content UTF8 adds a BOM, which is not valid JSON here.
  [IO.File]::WriteAllText($Steps, $CreateScript, [Text.UTF8Encoding]::new($false))
  [void] (Invoke-Cli @('run', '--root', $Work, '--script', $Steps))
  $Info = Invoke-Cli @('info', $Document) | ConvertFrom-Json
  if ($Info.pages -ne 1) { throw 'Packaged CLI did not save a readable PDF.' }
  $DocumentHash = (Get-FileHash $Document -Algorithm SHA256).Hash
  $App = Start-Process -FilePath $Executable -ArgumentList @('--new-window', '--control', "`"$Control`"", '--language', 'en', "`"$Document`"") -PassThru
  $Deadline = (Get-Date).AddSeconds(90)
  while (-not (Test-Path -LiteralPath $Control)) {
    if ($App.HasExited -or (Get-Date) -gt $Deadline) { throw 'Installed PDF failed to start its opt-in test control.' }
    Start-Sleep -Milliseconds 250
  }
  [void] (Invoke-Ui @('resize', 'width=1440', 'height=920'))
  $Ready = $false
  do {
    $State = Invoke-Ui @('state') | ConvertFrom-Json
    if ($State.active -and -not $State.pending_opens) { $Ready = $true; break }
    Start-Sleep -Milliseconds 250
  } while ((Get-Date) -lt $Deadline)
  if (-not $Ready) { throw 'Installed PDF did not finish opening the sample document.' }
  foreach ($Language in 'en', 'pt-BR', 'es') {
    [void] (Invoke-Ui @('set', 'key=language', "value=$Language"))
    $StateJson = Invoke-Ui @('state')
    $State = $StateJson | ConvertFrom-Json
    if ($State.language -ine $Language -or $State.active.page_errors.Count -ne 0) {
      throw "The installed PDF did not render the document in $Language."
    }
    $StateJson | Set-Content -LiteralPath (Join-Path $Evidence "state-$Language.json")
    $Screenshot = Join-Path $Evidence "pdf-$Language.png"
    [void] (Invoke-Ui @('screenshot', '--out', $Screenshot))
    if (-not (Test-Path $Screenshot) -or (Get-Item $Screenshot).Length -lt 10000) { throw "Missing native screenshot: $Language" }
  }
  [void] (Invoke-Ui @('quit'))
  if (-not $App.WaitForExit(30000) -or $App.ExitCode -ne 0) { throw 'Installed PDF did not exit normally.' }
  $App = $null

  # Update the real installed MSIX with the same files and a higher package version.
  $AppData = Join-Path $env:LOCALAPPDATA "Packages\$($Installed.PackageFamilyName)\LocalState"
  New-Item -ItemType Directory -Force -Path $AppData | Out-Null
  $Preference = Join-Path $AppData 'acceptance-preference.txt'
  Set-Content -LiteralPath $Preference -Value 'preserve preferences across Store updates'
  $MakeAppx = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin" -Recurse -Filter makeappx.exe |
    Where-Object FullName -Match '\\x64\\makeappx\.exe$' | Sort-Object FullName -Descending | Select-Object -First 1
  $UpgradeStage = Join-Path $Work 'upgrade'
  & $MakeAppx.FullName unpack /p $Package /d $UpgradeStage /o
  if ($LASTEXITCODE -ne 0) { throw 'Could not unpack validation MSIX.' }
  $UpgradeManifest = Join-Path $UpgradeStage 'AppxManifest.xml'
  [xml] $Xml = Get-Content -Raw $UpgradeManifest
  $CurrentVersion = [version] $Xml.Package.Identity.Version
  $Xml.Package.Identity.Version = "$($CurrentVersion.Major).$($CurrentVersion.Minor).$($CurrentVersion.Build + 1).0"
  $Xml.Save($UpgradeManifest)
  $Upgrade = Join-Path $Work 'upgrade.msix'
  & $MakeAppx.FullName pack /d $UpgradeStage /p $Upgrade /o
  if ($LASTEXITCODE -ne 0) { throw 'Could not create validation upgrade.' }
  Add-AppxPackage -Path $Upgrade -AllowUnsigned
  $Installed = Get-AppxPackage -Name $Metadata.identity_name
  if ($Installed.Version.ToString() -ne $Xml.Package.Identity.Version) { throw 'MSIX update was not applied.' }
  if ((Get-Content -LiteralPath $Preference) -cne 'preserve preferences across Store updates') { throw 'MSIX update removed app data.' }
  [void] (Invoke-Cli @('info', $Document))
} finally {
  if ($App -and -not $App.HasExited) { $App.Kill(); $App.WaitForExit() }
  $Registered = Get-AppxPackage -Name $Metadata.identity_name
  if ($Registered) { Remove-AppxPackage -Package $Registered.PackageFullName }
  Remove-Item -LiteralPath $Control -Force -ErrorAction SilentlyContinue
}
if (Get-AppxPackage -Name $Metadata.identity_name) { throw 'MSIX uninstall did not remove registration.' }
if ((Test-Path $Cli) -or (Test-Path $Executable)) { throw 'MSIX uninstall left its execution aliases behind.' }
if ((Get-FileHash $Document -Algorithm SHA256).Hash -cne $DocumentHash) { throw 'Installation lifecycle changed the user document.' }
if ((Get-ItemProperty -LiteralPath $ChoicePath -ErrorAction SilentlyContinue).ProgId -cne $BeforeChoice) { throw 'MSIX changed the default PDF handler.' }
@{
  status = 'passed'
  namespace = 'unsigned test namespace on disposable CI only'
  source_commit = $Metadata.source_commit
  package_sha256 = $Metadata.sha256
  checks = @('SDK manifest validation', 'MSIX install and file hashes', 'installed CLI save/reopen and OCR', 'native PDF GUI EN/PT/ES', 'MSIX upgrade preserves app data', 'uninstall preserves user document and default PDF handler')
  store_certification = 'pending'
} | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $Evidence 'result.json') -Encoding utf8
