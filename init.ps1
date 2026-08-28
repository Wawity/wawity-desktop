$ErrorActionPreference = 'Stop'
$root = 'C:\Users\dassa\Desktop\wawity'
$timestamp = Get-Date -Format 'yyyy-MM-dd_HH-mm'
$outFile = Join-Path $root "wawity_dump_$timestamp.txt"

$extensions = @('.rs', '.toml', '.json', '.ts', '.js', '.vue', '.html', '.css', '.md', '.java', '.kt', '.xml')
$excludeDirs = @('node_modules', 'target', 'dist', '.git', '.vscode', '.idea')

function Should-Exclude($path) {
    foreach ($dir in $excludeDirs) {
        if ($path -match "\\$dir\\|\\$dir$|^$dir\\|^$dir$") { return $true }
    }
    return $false
}

$files = Get-ChildItem -Path $root -Recurse -File | Where-Object {
    $extensions -contains $_.Extension -and
    -not (Should-Exclude $_.FullName) -and
    $_.Name -ne 'package-lock.json' -and
    $_.Name -ne 'Cargo.lock' -and
    $_.Name -ne 'pnpm-lock.yaml'
} | Sort-Object FullName

$sb = New-Object System.Text.StringBuilder
[void]$sb.AppendLine('=' * 80)
[void]$sb.AppendLine('WAWITY VPN PROJECT - FULL SOURCE DUMP')
[void]$sb.AppendLine("Generated: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')")
[void]$sb.AppendLine("Total files: $($files.Count)")
[void]$sb.AppendLine('=' * 80)
[void]$sb.AppendLine()
[void]$sb.AppendLine('# FILE TREE')
[void]$sb.AppendLine()

foreach ($f in $files) {
    $rel = $f.FullName.Substring($root.Length + 1)
    $size = if ($f.Length -gt 1024) { "$([math]::Round($f.Length / 1024, 1)) KB" } else { "$($f.Length) B" }
    [void]$sb.AppendLine("$rel  ($size)")
}

[void]$sb.AppendLine()

$langMap = @{
    '.rs' = 'rust'
    '.toml' = 'toml'
    '.json' = 'json'
    '.ts' = 'typescript'
    '.js' = 'javascript'
    '.vue' = 'vue'
    '.html' = 'html'
    '.css' = 'css'
    '.md' = 'markdown'
}

foreach ($f in $files) {
    $rel = $f.FullName.Substring($root.Length + 1).Replace('\', '/')
    $lang = $langMap[$f.Extension]
    $content = Get-Content -Path $f.FullName -Raw -Encoding UTF8 -ErrorAction SilentlyContinue
    
    [void]$sb.AppendLine('#' * 80)
    [void]$sb.AppendLine("FILE: $rel")
    [void]$sb.AppendLine("SIZE: $($f.Length) bytes")
    [void]$sb.AppendLine('#' * 80)
    [void]$sb.AppendLine()
    [void]$sb.AppendLine('```' + $lang)
    [void]$sb.AppendLine($content)
    [void]$sb.AppendLine('```')
    [void]$sb.AppendLine()
}

[void]$sb.AppendLine('=' * 80)
[void]$sb.AppendLine("END OF DUMP - Total files: $($files.Count)")
[void]$sb.AppendLine('=' * 80)

Set-Content -Path $outFile -Value $sb.ToString() -Encoding UTF8
$totalSize = (Get-Item $outFile).Length
$sizeMb = [math]::Round($totalSize / 1MB, 2)

Write-Host ''
Write-Host '===========================================' -ForegroundColor Green
Write-Host ' DUMP COMPLETE ' -ForegroundColor Cyan
Write-Host '===========================================' -ForegroundColor Green
Write-Host "File: $outFile" -ForegroundColor Yellow
Write-Host "Files included: $($files.Count)" -ForegroundColor White
Write-Host "Dump size: $sizeMb MB" -ForegroundColor White
Write-Host '===========================================' -ForegroundColor Green