# Called by Tauri bundler: sign.ps1 <path-to-binary>
# Signs the file if a certificate is configured via environment variables.
# Without configuration this script exits 0 so local builds skip signing.

param(
    [Parameter(Mandatory = $true, Position = 0)]
    [string]$FilePath
)

$ErrorActionPreference = 'Stop'

$pfxPath = $env:WAWITY_SIGN_PFX_PATH
$pfxPassword = $env:WAWITY_SIGN_PFX_PASSWORD

if (-not $pfxPath -or -not (Test-Path -LiteralPath $pfxPath)) {
    Write-Host "sign.ps1: no certificate configured (set WAWITY_SIGN_PFX_PATH / WAWITY_SIGN_PFX_PASSWORD), skipping signature for $FilePath"
    exit 0
}

$signtool = Get-Command signtool.exe -ErrorAction SilentlyContinue
if (-not $signtool) {
    $kitsRoot = 'C:\Program Files (x86)\Windows Kits\10\bin'
    if (Test-Path $kitsRoot) {
        $candidate = Get-ChildItem $kitsRoot -Directory |
            Sort-Object Name -Descending |
            ForEach-Object { Join-Path $_.FullName 'x64\signtool.exe' } |
            Where-Object { Test-Path $_ } |
            Select-Object -First 1
        if ($candidate) { $signtool = Get-Item $candidate }
    }
}
if (-not $signtool) {
    Write-Warning "sign.ps1: signtool.exe not found, skipping signature"
    exit 0
}

& $signtool.FullName sign /f $pfxPath /p $pfxPassword /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 $FilePath
if ($LASTEXITCODE -ne 0) {
    Write-Warning "sign.ps1: signtool failed with exit code $LASTEXITCODE, continuing without signature"
    exit 0
}

Write-Host "sign.ps1: signed $FilePath"
exit 0
