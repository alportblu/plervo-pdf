<# Actual application UI at Store listing resolution; native lifecycle remains a separate gate. #>
param([ValidateSet('video', 'cad')][string] $Product, [string] $Source = 'source')
$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$Source = (Resolve-Path $Source).Path
$Crate = if ($Product -eq 'video') { 'filmcraft-ui-egui' } else { 'cadcraft-ui-egui' }
$Examples = Join-Path $Source 'crates/ui-egui/examples'
New-Item -ItemType Directory -Force -Path $Examples | Out-Null
$Helper = Join-Path $Examples 'plervo_store_shot.rs'
if (Test-Path $Helper) { throw 'Listing helper must not overwrite a source file.' }
Copy-Item (Join-Path $PSScriptRoot "shot-$Product.rs") $Helper
$Output = Join-Path $Root "dist/validation/$Product/listing"
New-Item -ItemType Directory -Force -Path $Output | Out-Null
Push-Location $Source
try {
  cargo clippy --locked -p $Crate --example plervo_store_shot -- -D warnings
  if ($LASTEXITCODE -ne 0) { throw 'Listing helper lint failed.' }
  foreach ($Language in 'en', 'pt-br', 'es') {
    $Png = Join-Path $Output "$Language.png"
    cargo run --locked -p $Crate --example plervo_store_shot -- $Png $Language
    if ($LASTEXITCODE -ne 0) { throw "Listing render failed: $Language" }
  }
} finally {
  Pop-Location
  Remove-Item -LiteralPath $Helper
}
