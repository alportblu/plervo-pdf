<# Install, repair and remove our MSI only on a disposable GitHub runner. #>
param([Parameter(Mandatory)] [string] $Path)
$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_OS -ne 'Windows') {
  throw 'This acceptance test is restricted to disposable Windows GitHub runners.'
}
$Msi = (Resolve-Path -LiteralPath $Path).Path
$Install = Join-Path $env:ProgramFiles 'Plervo PDF'
if (Test-Path -LiteralPath $Install) { throw "Refusing to replace an existing installation: $Install" }
$Work = Join-Path $env:RUNNER_TEMP ('plervo-install-' + [Guid]::NewGuid())
$Evidence = Join-Path $PWD 'dist/validation'
New-Item -ItemType Directory -Force -Path $Work, $Evidence | Out-Null
$Sentinel = Join-Path $Work 'preserve.txt'
Set-Content -LiteralPath $Sentinel -Value 'Plervo user documents survive uninstall'
$ChoicePath = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.pdf\UserChoice'
$BeforeChoice = (Get-ItemProperty -LiteralPath $ChoicePath -ErrorAction SilentlyContinue).ProgId

function Invoke-Msi([string] $Operation, [string] $Label) {
  $Log = Join-Path $Evidence "windows-$Label.log"
  $Result = Start-Process msiexec.exe -ArgumentList @($Operation, "`"$Msi`"", '/qn', '/norestart', '/L*v', "`"$Log`"") -Wait -PassThru
  if ($Result.ExitCode -notin @(0, 3010)) { throw "MSI $Label failed: $($Result.ExitCode); see $Log" }
}
function Invoke-Cli([string[]] $CliArgs) {
  $Result = & (Join-Path $Install 'plervo-pdf-cli.exe') @CliArgs
  if ($LASTEXITCODE -ne 0) { throw "Installed CLI failed with exit $LASTEXITCODE" }
  return $Result
}
$Installed = $false
try {
  Invoke-Msi '/i' 'install'
  $Installed = $true
  if (-not (Test-Path (Join-Path $Install 'plervo-pdf.exe'))) { throw 'Installed app missing' }
  Invoke-Cli -CliArgs @('--version')
  $Ocr = Invoke-Cli -CliArgs @('run', 'ocr_status') | ConvertFrom-Json
  if (-not $Ocr.available) { throw 'Installed CLI cannot find OCR models' }
  $Script = Join-Path $Work 'steps.json'
  @'
[{"tool":"doc_create","args":{"from":"text","text":"Plervo installation test"}},
 {"tool":"doc_save","args":{"doc":1,"path":"saved.pdf"}}]
'@ | Set-Content -LiteralPath $Script -Encoding utf8
  Invoke-Cli -CliArgs @('run', '--root', $Work, '--script', $Script)
  $Info = Invoke-Cli -CliArgs @('info', (Join-Path $Work 'saved.pdf')) | ConvertFrom-Json
  if ($Info.pages -ne 1) { throw 'Installed CLI did not save a readable PDF' }
  Invoke-Msi '/fa' 'repair'
  Invoke-Cli -CliArgs @('--version')
} finally {
  if ($Installed) { Invoke-Msi '/x' 'uninstall' }
}
if (Test-Path (Join-Path $Install 'plervo-pdf.exe')) { throw 'Uninstall left the app installed' }
if (Test-Path (Join-Path $Install 'plervo-pdf-cli.exe')) { throw 'Uninstall left the CLI installed' }
if ((Get-Content -LiteralPath $Sentinel) -ne 'Plervo user documents survive uninstall') { throw 'User sentinel changed' }
if (-not (Test-Path (Join-Path $Work 'saved.pdf'))) { throw 'Uninstall removed the saved PDF' }
$AfterChoice = (Get-ItemProperty -LiteralPath $ChoicePath -ErrorAction SilentlyContinue).ProgId
if ($BeforeChoice -cne $AfterChoice) { throw 'Installer changed the default PDF application' }
'MSI install, installed CLI save/reopen/OCR discovery, repair, uninstall, document/default-handler preservation passed.' |
  Set-Content -LiteralPath (Join-Path $Evidence 'windows-install-result.txt')
