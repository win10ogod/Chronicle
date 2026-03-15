param(
  [string]$Target = "",
  [string]$OutDir = "dist",
  [switch]$NoVerify
)

$ErrorActionPreference = "Stop"

function Write-Utf8NoBom([string]$Path, [string]$Content) {
  $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
  [System.IO.File]::WriteAllText($Path, $Content, $utf8NoBom)
}

$root = Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $root

$versionLine = Select-String -LiteralPath (Join-Path $root "Cargo.toml") -Pattern '^\s*version\s*=\s*"([^"]+)"\s*$' | Select-Object -First 1
if (-not $versionLine) { throw "Unable to read version from Cargo.toml" }
$version = $versionLine.Matches[0].Groups[1].Value

if (-not $NoVerify) {
  cargo fmt --check
  cargo clippy --all-targets -- -D warnings
  cargo test --locked
}

if (-not $env:CARGO_TARGET_DIR) {
  $env:CARGO_TARGET_DIR = "target-win"
}

if ($Target -ne "") {
  cargo build --release --locked --target $Target
  $bin = Join-Path $root "$($env:CARGO_TARGET_DIR)\$Target\release\chronicle.exe"
  $triple = $Target
} else {
  cargo build --release --locked
  $bin = Join-Path $root "$($env:CARGO_TARGET_DIR)\release\chronicle.exe"
  $hostLine = (rustc -Vv | Select-String -Pattern '^host:\s+').ToString()
  $triple = $hostLine.Split()[-1]
}

if (-not (Test-Path -LiteralPath $bin)) { throw "Binary not found: $bin" }

$dist = Join-Path $root $OutDir
$stage = Join-Path $dist "stage"
$pkgName = "chronicle-v$version-$triple"
$pkgDir = Join-Path $stage $pkgName

Remove-Item -Recurse -Force $pkgDir -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path (Join-Path $pkgDir "completions") | Out-Null

Copy-Item -LiteralPath $bin -Destination (Join-Path $pkgDir "chronicle.exe") -Force
Copy-Item -LiteralPath (Join-Path $root "README.md") -Destination (Join-Path $pkgDir "README.md") -Force

foreach ($shell in @("bash","zsh","fish","powershell","elvish")) {
  $ext = $shell
  if ($shell -eq "powershell") { $ext = "ps1" }
  $content = & $bin completions $shell
  Write-Utf8NoBom (Join-Path $pkgDir "completions\chronicle.$ext") $content
}

New-Item -ItemType Directory -Force -Path $dist | Out-Null
$zipPath = Join-Path $dist "$pkgName.zip"
Remove-Item -Force $zipPath -ErrorAction SilentlyContinue

Push-Location $stage
try {
  Compress-Archive -Path $pkgName -DestinationPath $zipPath -Force
} finally {
  Pop-Location
}

$hash = (Get-FileHash -LiteralPath $zipPath -Algorithm SHA256).Hash.ToLowerInvariant()
Write-Utf8NoBom (Join-Path $dist "$pkgName.zip.sha256") "$hash  $([IO.Path]::GetFileName($zipPath))`n"

Write-Host "Wrote:"
Write-Host "  $zipPath"
Write-Host "  $zipPath.sha256"
