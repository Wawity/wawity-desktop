param(
  [string]$ProjectRoot = "..",
  [string]$Configuration = "release",
  [switch]$NoPause
)

$ErrorActionPreference = "Stop"

try {
  $releaseDir = Join-Path $ProjectRoot "src-tauri\target\$Configuration"
  $tauriDir = Join-Path $ProjectRoot "src-tauri"

  if (!(Test-Path $releaseDir)) {
    throw "Папка $releaseDir не найдена. Сначала собери приложение: npm run tauri build"
  }

  $appExe = $null
  foreach ($candidate in @("Wawity.exe", "wawity.exe")) {
    $probe = Join-Path $releaseDir $candidate
    if (Test-Path $probe) {
      $appExe = $probe
      break
    }
  }
  if (-not $appExe) {
    $found = Get-ChildItem $releaseDir -Filter "*.exe" -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Name
    throw "Не нашёл Wawity.exe / wawity.exe в $releaseDir. Есть там: $($found -join ', '). Собери приложение: npm run tauri build"
  }

  $needed = @(
    (Join-Path $tauriDir "binaries\sing-box-x86_64.exe"),
    (Join-Path $tauriDir "binaries\wintun.dll"),
    (Join-Path $tauriDir "rulesets\geosite-category-ads-all.srs"),
    (Join-Path $tauriDir "rulesets\geosite-private.srs")
  )
  foreach ($item in $needed) {
    if (!(Test-Path $item)) {
      throw "Не найден файл ресурсов: $item"
    }
  }

  New-Item -ItemType Directory -Force -Path "payload" | Out-Null

  $stage = Join-Path $env:TEMP "wawity_stage"
  Remove-Item $stage -Recurse -Force -ErrorAction SilentlyContinue
  New-Item -ItemType Directory -Force -Path $stage | Out-Null
  New-Item -ItemType Directory -Force -Path (Join-Path $stage "binaries") | Out-Null
  New-Item -ItemType Directory -Force -Path (Join-Path $stage "rulesets") | Out-Null

  Copy-Item $appExe (Join-Path $stage "Wawity.exe")
  Copy-Item (Join-Path $tauriDir "binaries\sing-box-x86_64.exe") (Join-Path $stage "binaries")
  Copy-Item (Join-Path $tauriDir "binaries\wintun.dll") (Join-Path $stage "binaries")
  Copy-Item (Join-Path $tauriDir "rulesets\geosite-category-ads-all.srs") (Join-Path $stage "rulesets")
  Copy-Item (Join-Path $tauriDir "rulesets\geosite-private.srs") (Join-Path $stage "rulesets")

  Remove-Item "payload\app.zip" -Force -ErrorAction SilentlyContinue
  Compress-Archive -Path (Join-Path $stage "*") -DestinationPath "payload\app.zip" -CompressionLevel Optimal
  Remove-Item $stage -Recurse -Force

  if (!(Test-Path "payload\MicrosoftEdgeWebView2Setup.exe")) {
    Write-Host "Скачиваю WebView2 bootstrapper..."
    Invoke-WebRequest "https://go.microsoft.com/fwlink/p/?LinkId=2124703" -OutFile "payload\MicrosoftEdgeWebView2Setup.exe"
  }

  $zipInfo = Get-Item "payload\app.zip"
  Write-Host ""
  Write-Host "OK: payload/app.zip готов ($([math]::Round($zipInfo.Length / 1MB, 1)) MB)" -ForegroundColor Green
}
catch {
  Write-Host ""
  Write-Host "ОШИБКА: $($_.Exception.Message)" -ForegroundColor Red
  if (-not $NoPause) {
    Write-Host ""
    Read-Host "Нажми Enter чтобы закрыть"
  }
  exit 1
}

if (-not $NoPause) {
  Write-Host ""
  Read-Host "Нажми Enter чтобы закрыть"
}
exit 0
