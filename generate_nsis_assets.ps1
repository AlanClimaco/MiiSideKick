Add-Type -AssemblyName System.Drawing

$projectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
if (-not $projectRoot) { $projectRoot = Get-Location }

$iconsDir = Join-Path $projectRoot "src-tauri\icons"
$reacoesDir = Join-Path $projectRoot "src\assets\reacoes"

$miiFelizPath = Join-Path $reacoesDir "mii-feliz.png"
$miiFrustradoPath = Join-Path $reacoesDir "mii-frustrado.png"

# Helper to draw linear gradient
function Create-SidebarBitmap([string]$avatarPath, [string]$title, [string]$subtitle, [System.Drawing.Color]$colTop, [System.Drawing.Color]$colMid, [System.Drawing.Color]$colBottom, [string]$outputPath) {
    $bmp = New-Object System.Drawing.Bitmap(164, 314, [System.Drawing.Imaging.PixelFormat]::Format24bppRgb)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $g.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::ClearTypeGridFit

    # Background gradient
    $rectTop = New-Object System.Drawing.Rectangle(0, 0, 164, 160)
    $brushTop = New-Object System.Drawing.Drawing2D.LinearGradientBrush($rectTop, $colTop, $colMid, [System.Drawing.Drawing2D.LinearGradientMode]::Vertical)
    $g.FillRectangle($brushTop, $rectTop)
    $brushTop.Dispose()

    $rectBottom = New-Object System.Drawing.Rectangle(0, 159, 164, 155)
    $brushBottom = New-Object System.Drawing.Drawing2D.LinearGradientBrush($rectBottom, $colMid, $colBottom, [System.Drawing.Drawing2D.LinearGradientMode]::Vertical)
    $g.FillRectangle($brushBottom, $rectBottom)
    $brushBottom.Dispose()

    # Soft glowing circle behind Mii
    $glowRect = New-Object System.Drawing.Rectangle(22, 45, 120, 120)
    $glowBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(40, 255, 255, 255))
    $g.FillEllipse($glowBrush, $glowRect)
    $glowBrush.Dispose()

    # Draw Avatar
    if (Test-Path $avatarPath) {
        $avatar = [System.Drawing.Image]::FromFile($avatarPath)
        $avatarRect = New-Object System.Drawing.Rectangle(27, 45, 110, 110)
        $g.DrawImage($avatar, $avatarRect)
        $avatar.Dispose()
    }

    # Separator line with gloss
    $penWhite = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(180, 255, 255, 255), 1)
    $penShadow = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(60, 0, 40, 80), 1)
    $g.DrawLine($penShadow, 20, 185, 144, 185)
    $g.DrawLine($penWhite, 20, 186, 144, 186)
    $penWhite.Dispose()
    $penShadow.Dispose()

    # Text Branding
    $fontFamily = New-Object System.Drawing.FontFamily("Segoe UI")
    $titleFont = New-Object System.Drawing.Font($fontFamily, 13, [System.Drawing.FontStyle]::Bold, [System.Drawing.GraphicsUnit]::Point)
    $subFont = New-Object System.Drawing.Font($fontFamily, 9, [System.Drawing.FontStyle]::Regular, [System.Drawing.GraphicsUnit]::Point)

    $format = New-Object System.Drawing.StringFormat
    $format.Alignment = [System.Drawing.StringAlignment]::Center

    $textBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(25, 45, 75))
    $subBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(70, 95, 125))

    $g.DrawString($title, $titleFont, $textBrush, 82, 200, $format)
    $g.DrawString($subtitle, $subFont, $subBrush, (New-Object System.Drawing.RectangleF(12, 228, 140, 60)), $format)

    $titleFont.Dispose()
    $subFont.Dispose()
    $fontFamily.Dispose()
    $textBrush.Dispose()
    $subBrush.Dispose()
    $format.Dispose()
    $g.Dispose()

    $bmp.Save($outputPath, [System.Drawing.Imaging.ImageFormat]::Bmp)
    $bmp.Dispose()
    Write-Output "Created: $outputPath"
}

# Header Bitmap (150x57)
function Create-HeaderBitmap([string]$avatarPath, [System.Drawing.Color]$colLeft, [System.Drawing.Color]$colRight, [string]$outputPath) {
    $bmp = New-Object System.Drawing.Bitmap(150, 57, [System.Drawing.Imaging.PixelFormat]::Format24bppRgb)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic

    $rect = New-Object System.Drawing.Rectangle(0, 0, 150, 57)
    $brush = New-Object System.Drawing.Drawing2D.LinearGradientBrush($rect, $colLeft, $colRight, [System.Drawing.Drawing2D.LinearGradientMode]::Horizontal)
    $g.FillRectangle($brush, $rect)
    $brush.Dispose()

    if (Test-Path $avatarPath) {
        $avatar = [System.Drawing.Image]::FromFile($avatarPath)
        $avatarRect = New-Object System.Drawing.Rectangle(98, 4, 48, 48)
        $g.DrawImage($avatar, $avatarRect)
        $avatar.Dispose()
    }

    $g.Dispose()
    $bmp.Save($outputPath, [System.Drawing.Imaging.ImageFormat]::Bmp)
    $bmp.Dispose()
    Write-Output "Created: $outputPath"
}

# Generate ICO helper
function Create-IcoFromPng([string]$sourcePng, [string]$outputIco) {
    $sizes = @(16, 32, 48, 64, 128)
    $tempPngs = @()

    $src = [System.Drawing.Image]::FromFile($sourcePng)
    foreach ($sz in $sizes) {
        $resized = New-Object System.Drawing.Bitmap($sz, $sz, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
        $g = [System.Drawing.Graphics]::FromImage($resized)
        $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
        $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
        $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
        $g.Clear([System.Drawing.Color]::Transparent)
        $g.DrawImage($src, 0, 0, $sz, $sz)
        $g.Dispose()

        $tempFile = [System.IO.Path]::GetTempFileName() + ".png"
        $resized.Save($tempFile, [System.Drawing.Imaging.ImageFormat]::Png)
        $resized.Dispose()
        $tempPngs += $tempFile
    }
    $src.Dispose()

    $ms = New-Object System.IO.MemoryStream
    $bw = New-Object System.IO.BinaryWriter($ms)
    $bw.Write([uint16]0)
    $bw.Write([uint16]1)
    $bw.Write([uint16]$tempPngs.Length)

    $offset = 6 + ($tempPngs.Length * 16)
    $bytesList = @()

    for ($i = 0; $i -lt $tempPngs.Length; $i++) {
        $bytes = [System.IO.File]::ReadAllBytes($tempPngs[$i])
        $bytesList += ,$bytes
        $sz = $sizes[$i]
        $w = if ($sz -ge 256) { [byte]0 } else { [byte]$sz }
        $h = if ($sz -ge 256) { [byte]0 } else { [byte]$sz }

        $bw.Write($w)
        $bw.Write($h)
        $bw.Write([byte]0)
        $bw.Write([byte]0)
        $bw.Write([uint16]1)
        $bw.Write([uint16]32)
        $bw.Write([uint32]$bytes.Length)
        $bw.Write([uint32]$offset)

        $offset += $bytes.Length
    }

    foreach ($b in $bytesList) {
        $bw.Write($b)
    }

    [System.IO.File]::WriteAllBytes($outputIco, $ms.ToArray())
    $bw.Dispose()
    $ms.Dispose()

    foreach ($t in $tempPngs) {
        Remove-Item $t -Force
    }

    Write-Output "Created: $outputIco"
}

# 1. Installer Sidebar (Mii Feliz / Frutiger Aero)
Create-SidebarBitmap `
    -avatarPath $miiFelizPath `
    -title "MiiSideKick" `
    -subtitle "Seu Assistente Amigável e Inteligente" `
    -colTop ([System.Drawing.Color]::FromArgb(255, 255, 255)) `
    -colMid ([System.Drawing.Color]::FromArgb(224, 242, 254)) `
    -colBottom ([System.Drawing.Color]::FromArgb(186, 230, 253)) `
    -outputPath (Join-Path $iconsDir "nsis-sidebar-install.bmp")

# 2. Uninstaller Sidebar (Mii Frustrado)
Create-SidebarBitmap `
    -avatarPath $miiFrustradoPath `
    -title "MiiSideKick" `
    -subtitle "Assistente de Desinstalação" `
    -colTop ([System.Drawing.Color]::FromArgb(255, 255, 255)) `
    -colMid ([System.Drawing.Color]::FromArgb(241, 245, 249)) `
    -colBottom ([System.Drawing.Color]::FromArgb(203, 213, 225)) `
    -outputPath (Join-Path $iconsDir "nsis-sidebar-uninstall.bmp")

# 3. Header Bitmaps
Create-HeaderBitmap `
    -avatarPath $miiFelizPath `
    -colLeft ([System.Drawing.Color]::FromArgb(255, 255, 255)) `
    -colRight ([System.Drawing.Color]::FromArgb(224, 242, 254)) `
    -outputPath (Join-Path $iconsDir "nsis-header-install.bmp")

Create-HeaderBitmap `
    -avatarPath $miiFrustradoPath `
    -colLeft ([System.Drawing.Color]::FromArgb(255, 255, 255)) `
    -colRight ([System.Drawing.Color]::FromArgb(241, 245, 249)) `
    -outputPath (Join-Path $iconsDir "nsis-header-uninstall.bmp")

# 4. Uninstaller Icon (.ico)
Create-IcoFromPng `
    -sourcePng $miiFrustradoPath `
    -outputIco (Join-Path $iconsDir "uninstall.ico")

Write-Output "All NSIS visual assets generated successfully!"
