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

$productInfo = @{}
foreach ($file in $yamlFiles) {
    $content = Get-Content -LiteralPath $file.FullName
    $code = ($content | Where-Object { $_ -match '^code:\s*(.+)$' } | Select-Object -First 1) -replace '^code:\s*', ''
    $name = ($content | Where-Object { $_ -match '^name:\s*(.+)$' } | Select-Object -First 1) -replace '^name:\s*', ''
    $price = ($content | Where-Object { $_ -match '^price:\s*(.+)$' } | Select-Object -First 1) -replace '^price:\s*', ''
    if ($code) {
        $productInfo[$code.Trim()] = [pscustomobject]@{
            Name  = $name.Trim()
            Price = $price.Trim()
        }
    }
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

function ConvertTo-QrPdf {
    param([string]$qrDir, [hashtable]$productInfo)
    Add-Type -AssemblyName System.Drawing

    $pngFiles = Get-ChildItem -LiteralPath $qrDir -Filter *.png | Sort-Object Name
    if ($pngFiles.Count -eq 0) {
        Write-Host "No QR codes to stitch into PDF." -ForegroundColor Yellow
        return
    }

    function Escape-PdfText {
        param([string]$s)
        return ($s -replace '\\', '\\\\') -replace '\(', '\(' -replace '\)', '\)'
    }

    function Measure-PdfText {
        param([string]$text, [bool]$bold, [int]$fontSize)
        $units = 0
        foreach ($ch in $text.ToCharArray()) {
            $units += Get-CharWidth -ch $ch -bold $bold
        }
        return $units * $fontSize / 1000.0
    }

    function Get-CharWidth {
        param([char]$ch, [bool]$bold)
        $b = $ch -eq 'B' -or $bold
        switch -Regex ($ch.ToString()) {
            '^[0-9]$' { if ($bold) { return 556 } else { return 556 } }
            '^[cfsiIt]$' { if ($bold) { return 333 } else { return 278 } }
            '^[a]$' { if ($bold) { return 556 } else { return 556 } }
            '^[j]$' { if ($bold) { return 278 } else { return 222 } }
            '^[l]$' { if ($bold) { return 278 } else { return 222 } }
            '^[n]$' { if ($bold) { return 611 } else { return 556 } }
            '^[m]$' { if ($bold) { return 889 } else { return 833 } }
            '^[r]$' { if ($bold) { return 389 } else { return 333 } }
            '^[s]$' { if ($bold) { return 556 } else { return 500 } }
            '^[b]$' { if ($bold) { return 611 } else { return 556 } }
            '^[d]$' { if ($bold) { return 611 } else { return 556 } }
            '^[e]$' { if ($bold) { return 556 } else { return 556 } }
            '^[g]$' { if ($bold) { return 611 } else { return 556 } }
            '^[h]$' { if ($bold) { return 611 } else { return 556 } }
            '^[o]$' { if ($bold) { return 611 } else { return 556 } }
            '^[p]$' { if ($bold) { return 611 } else { return 556 } }
            '^[q]$' { if ($bold) { return 611 } else { return 556 } }
            '^[u]$' { if ($bold) { return 611 } else { return 556 } }
            '^[v]$' { if ($bold) { return 556 } else { return 500 } }
            '^[w]$' { if ($bold) { return 778 } else { return 722 } }
            '^[x]$' { if ($bold) { return 556 } else { return 500 } }
            '^[y]$' { if ($bold) { return 556 } else { return 500 } }
            '^[z]$' { if ($bold) { return 500 } else { return 500 } }
            '^[A-Z]$' {
                $map = @{ 'A' = 722; 'B' = 722; 'C' = 722; 'D' = 722; 'E' = 667; 'F' = 611; 'G' = 778; 'H' = 722;
                    'I' = 278; 'J' = 556; 'K' = 722; 'L' = 611; 'M' = 833; 'N' = 722; 'O' = 778; 'P' = 667;
                    'Q' = 778; 'R' = 722; 'S' = 667; 'T' = 611; 'U' = 722; 'V' = 667; 'W' = 944; 'X' = 667;
                    'Y' = 667; 'Z' = 611 }
                if ($bold) { return $map[$ch.ToString()] }
                $map2 = @{ 'A' = 667; 'B' = 667; 'C' = 722; 'D' = 722; 'E' = 667; 'F' = 611; 'G' = 778; 'H' = 722;
                    'I' = 278; 'J' = 500; 'K' = 667; 'L' = 556; 'M' = 833; 'N' = 722; 'O' = 778; 'P' = 667;
                    'Q' = 778; 'R' = 722; 'S' = 667; 'T' = 611; 'U' = 722; 'V' = 667; 'W' = 944; 'X' = 667;
                    'Y' = 667; 'Z' = 611 }
                return $map2[$ch.ToString()]
            }
            default {
                $d = @{ ' ' = 278; '$' = 556; '(' = 333; ')' = 333; ',' = 278; '-' = 333; '.' = 278; '/' = 278;
                    ':' = 278; ';' = 278; '&' = 667; '%' = 889; '+' = 584; '=' = 584; '_' = 556; '?' = 556 }
                $db = @{ ' ' = 278; '$' = 556; '(' = 333; ')' = 333; ',' = 278; '-' = 333; '.' = 278; '/' = 278;
                    ':' = 333; ';' = 333; '&' = 722; '%' = 889; '+' = 584; '=' = 584; '_' = 556; '?' = 611 }
                $t = $ch.ToString()
                if ($bold) { if ($db.ContainsKey($t)) { return $db[$t] } else { return 556 } }
                if ($d.ContainsKey($t)) { return $d[$t] }
                return 556
            }
        }
    }

    $images = @()
    foreach ($f in $pngFiles) {
        $info = $productInfo[$f.BaseName]
        $img = [System.Drawing.Image]::FromFile($f.FullName)
        $ms = New-Object System.IO.MemoryStream
        $img.Save($ms, [System.Drawing.Imaging.ImageFormat]::Jpeg)
        $images += [pscustomobject]@{
            Code   = $f.BaseName
            Name   = $(if ($info) { $info.Name } else { $f.BaseName })
            Price  = $(if ($info -and $info.Price) { "$($info.Price)" } else { "" })
            Width  = $img.Width
            Height = $img.Height
            Data   = $ms.ToArray()
        }
        $ms.Dispose()
        $img.Dispose()
    }

    $pageW = 612
    $pageH = 792
    $margin = 40
    $gap = 24
    $cols = 3
    $perPage = 9
    $qrSize = 150
    $cellW = ($pageW - 2 * $margin - $gap) / $cols
    $cellH = ($pageH - 2 * $margin - $gap) / 3
    $nPages = [math]::Ceiling($images.Count / $perPage)

    $script:pdf = New-Object System.IO.MemoryStream
    $script:offsets = [System.Collections.Generic.List[long]]::new()
    $script:offsets.Add(0)

    function Write-PdfText {
        param([string]$s)
        $b = [System.Text.Encoding]::ASCII.GetBytes($s)
        $script:pdf.Write($b, 0, $b.Length)
    }

    function Write-PdfObject {
        param(
            [int]$id,
            [string]$dict,
            [byte[]]$streamData = $null
        )
        while ($script:offsets.Count -le $id) {
            $script:offsets.Add(-1)
        }
        $script:offsets[$id] = $script:pdf.Position
        Write-PdfText "$id 0 obj`n"
        Write-PdfText $dict
        if ($null -ne $streamData) {
            Write-PdfText "stream`n"
            $script:pdf.Write($streamData, 0, $streamData.Length)
            Write-PdfText "`nendstream`n"
        }
        Write-PdfText "endobj`n"
    }

    $fontId = 3
    $fontBoldId = 4
    $pageStart = $fontBoldId + 1
    $contentStart = $pageStart + $nPages
    $imageStart = $contentStart + $nPages

    $imageIds = @()
    for ($i = 0; $i -lt $images.Count; $i++) {
        $imageIds += ($imageStart + $i)
    }

    Write-PdfText "%PDF-1.4`n"

    Write-PdfObject 1 "<< /Type /Catalog /Pages 2 0 R >>"

    $pageIds = @()
    for ($p = 0; $p -lt $nPages; $p++) {
        $pageIds += ($pageStart + $p)
    }
    $kids = ($pageIds | ForEach-Object { "$_ 0 R" }) -join " "
    Write-PdfObject 2 "<< /Type /Pages /Kids [$kids] /Count $nPages >>"

    Write-PdfObject $fontId "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>"
    Write-PdfObject $fontBoldId "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold >>"

    $imgCounter = 0
    foreach ($img in $images) {
        $streamDict = "<< /Type /XObject /Subtype /Image /Width $($img.Width) /Height $($img.Height) " +
            "/ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /DCTDecode /Length $($img.Data.Length) >>"
        Write-PdfObject $imageIds[$imgCounter] $streamDict $img.Data
        $imgCounter++
    }
    for ($p = 0; $p -lt $nPages; $p++) {
        $contentBuilder = New-Object System.Text.StringBuilder
        $usedIds = @()
        for ($k = 0; $k -lt $perPage; $k++) {
            $idx = $p * $perPage + $k
            if ($idx -ge $images.Count) { break }
            $usedIds += $imageIds[$idx]

            $col = $k % $cols
            $row = [math]::Floor($k / $cols)
            $x = $margin + $col * ($cellW + $gap) + ($cellW - $qrSize) / 2
            $yTop = $pageH - $margin - $row * ($cellH + $gap)
            $y = $yTop - $qrSize

            [void]$contentBuilder.AppendLine("q")
            [void]$contentBuilder.AppendLine("1 0 0 1 $x $y cm")
            [void]$contentBuilder.AppendLine("$qrSize 0 0 $qrSize 0 0 cm")
            [void]$contentBuilder.AppendLine("/Im$($imageIds[$idx]) Do")
            [void]$contentBuilder.AppendLine("Q")

            $name = Escape-PdfText -s $images[$idx].Name
            $price = Escape-PdfText -s "`$$($images[$idx].Price)"

            $cellLeft = $margin + $col * ($cellW + $gap)
            $cellCenter = $cellLeft + $cellW / 2

            $nameWidth = Measure-PdfText -text $name -bold $true -fontSize 18
            $priceWidth = Measure-PdfText -text $price -bold $false -fontSize 20

            $nameX = $cellCenter - $nameWidth / 2
            $priceX = $cellCenter - $priceWidth / 2

            $labelY = $y - 12
            [void]$contentBuilder.AppendLine("BT /F2 18 Tf $nameX $labelY Td ($name) Tj ET")
            [void]$contentBuilder.AppendLine("BT /F1 20 Tf $priceX $($labelY - 30) Td ($price) Tj ET")
        }

        $contentBytes = [System.Text.Encoding]::ASCII.GetBytes($contentBuilder.ToString())
        $contentDict = "<< /Length $($contentBytes.Length) >>"
        Write-PdfObject ($contentStart + $p) $contentDict $contentBytes

        $xobjects = ($usedIds | ForEach-Object { "/Im$_ $_ 0 R" }) -join " "
        $pageDict = "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 $pageW $pageH] " +
            "/Resources << /Font << /F1 $fontId 0 R /F2 $fontBoldId 0 R >> /XObject << $xobjects >> >> " +
            "/Contents $($contentStart + $p) 0 R >>"
        Write-PdfObject ($pageStart + $p) $pageDict
    }

    $maxId = $imageStart + $images.Count - 1
    $pdfBytes = $script:pdf.ToArray()
    $script:pdf.Dispose()

    $out = New-Object System.IO.MemoryStream
    $out.Write($pdfBytes, 0, $pdfBytes.Length)
    $b = [System.Text.Encoding]::ASCII.GetBytes("xref`n0 $($maxId + 1)`n")
    $out.Write($b, 0, $b.Length)
    $b = [System.Text.Encoding]::ASCII.GetBytes("0000000000 65535 f `n")
    $out.Write($b, 0, $b.Length)
    for ($i = 1; $i -le $maxId; $i++) {
        $entry = $offsets[$i].ToString().PadLeft(10, '0')
        $b = [System.Text.Encoding]::ASCII.GetBytes("$entry 00000 n `n")
        $out.Write($b, 0, $b.Length)
    }
    $xrefPos = $pdfBytes.Length
    $trailer = "trailer`n<< /Size $($maxId + 1) /Root 1 0 R >>`nstartxref`n$xrefPos`n%%EOF"
    $b = [System.Text.Encoding]::ASCII.GetBytes($trailer)
    $out.Write($b, 0, $b.Length)

    $pdfPath = Join-Path $qrDir "All-QR-Codes.pdf"
    [System.IO.File]::WriteAllBytes($pdfPath, $out.ToArray())
    $out.Dispose()

    Write-Host "Stitched $($images.Count) QR codes into $pdfPath" -ForegroundColor Green
}

ConvertTo-QrPdf -qrDir $qrDir -productInfo $productInfo