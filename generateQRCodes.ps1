$ErrorActionPreference = 'Stop'

$baseUrl = "https://www.4ambertechel.com"
$qrDir = Join-Path $PSScriptRoot "QR codes"
$productsDir = Join-Path $PSScriptRoot "products"
$maxRetries = 5
$retryDelayMs = 3000
$requestDelayMs = 3000

function Test-PngFile {
    param([string]$path)
    try {
        $bytes = [System.IO.File]::ReadAllBytes($path)
        return $bytes.Length -ge 8 -and
            $bytes[0] -eq 0x89 -and $bytes[1] -eq 0x50 -and
            $bytes[2] -eq 0x4E -and $bytes[3] -eq 0x47
    }
    catch {
        return $false
    }
}

if (-not (Test-Path $productsDir)) {
    Write-Host "products directory not found: $productsDir" -ForegroundColor Red
    exit 1
}

if (-not (Test-Path $qrDir)) {
    New-Item -ItemType Directory -Path $qrDir | Out-Null
}

$yamlFiles = Get-ChildItem -LiteralPath $productsDir -File -Filter *.yaml
if ($yamlFiles.Count -eq 0) {
    Write-Host "No product YAML files found in $productsDir" -ForegroundColor Yellow
    exit 0
}

foreach ($file in $yamlFiles) {
    $codeLine = Select-String -LiteralPath $file.FullName -Pattern '^code:\s*(.+)$' | Select-Object -First 1
    if (-not $codeLine) {
        Write-Host "Skipping $($file.Name): no code field" -ForegroundColor Yellow
        continue
    }

    $code = $codeLine.Matches[0].Groups[1].Value.Trim()
    $url = "$baseUrl/order/$code"
    $outputFile = Join-Path $qrDir "$code.png"

    if (Test-PngFile -path $outputFile) {
        Write-Host "Skipping $code (QR code already exists)" -ForegroundColor Gray
        continue
    }

    $payload = @{
        data = $url
        size = 200
        file = "png"
        download = $false
        config = @{
            body = "japnese"
            eye = "frame14"
            eyeBall = "ball16"
            bodyColor = "#FF0000"
            bgColor = "#FFFFFF"
            eye1Color = "#FF0000"
            eye2Color = "#FF0000"
            eye3Color = "#FF0000"
            eyeBall1Color = "#FF0000"
            eyeBall2Color = "#FF0000"
            eyeBall3Color = "#FF0000"
        }
    } | ConvertTo-Json -Depth 5

    $success = $false
    for ($attempt = 1; $attempt -le $maxRetries; $attempt++) {
        try {
            Invoke-WebRequest -Uri "https://api.qrcode-monkey.com/qr/custom" -Method Post -ContentType "application/json" -Body $payload -OutFile $outputFile -UseBasicParsing

            if (Test-PngFile -path $outputFile) {
                $success = $true
                break
            }
            Write-Host "  Retry ${attempt}: received non-PNG response (rate limited?)" -ForegroundColor Yellow
        }
        catch {
            Write-Host "  Retry ${attempt}: $($_.Exception.Message)" -ForegroundColor Yellow
        }
        Start-Sleep -Milliseconds $retryDelayMs
    }

    if ($success) {
        Write-Host "Generated QR code for $code -> $outputFile" -ForegroundColor Green
    }
    else {
        Write-Host "Failed to generate QR code for $code after $maxRetries attempts" -ForegroundColor Red
    }

    Start-Sleep -Milliseconds $requestDelayMs
}

Write-Host "`nQR code generation complete. Files saved in: $qrDir"