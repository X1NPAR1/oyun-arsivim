# Oyun Arşivim logosunu PNG olarak çizer (SVG kullanılmaz — proje kuralı).
# Konsept: grafit yuvarlak kare üzerinde, hafif açılı üst üste dizilmiş üç oyun kutusu ("arşiv").
# Çıktı: assets/brand/logo-1024.png (+ arayüz için 1x/2x/3x). Tauri ikonları: `pnpm tauri icon`.
param([string] $OutDir = (Join-Path $PSScriptRoot '..\assets\brand'))
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
New-Item -ItemType Directory -Force $OutDir | Out-Null

function New-RoundRect([float] $x, [float] $y, [float] $w, [float] $h, [float] $r) {
  $p = New-Object System.Drawing.Drawing2D.GraphicsPath
  $d = $r * 2
  $p.AddArc($x, $y, $d, $d, 180, 90)
  $p.AddArc($x + $w - $d, $y, $d, $d, 270, 90)
  $p.AddArc($x + $w - $d, $y + $h - $d, $d, $d, 0, 90)
  $p.AddArc($x, $y + $h - $d, $d, $d, 90, 90)
  $p.CloseFigure()
  return $p
}

function Draw-Logo([int] $size, [string] $file, [bool] $withTile = $true) {
  $bmp = New-Object System.Drawing.Bitmap $size, $size, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.SmoothingMode = 'AntiAlias'; $g.PixelOffsetMode = 'HighQuality'; $g.CompositingQuality = 'HighQuality'
  $g.Clear([System.Drawing.Color]::Transparent)
  $s = $size / 1024.0

  if ($withTile) {
    # Zemin: grafit yuvarlak kare, üstten çok hafif aydınlık.
    $tile = New-RoundRect (40 * $s) (40 * $s) (944 * $s) (944 * $s) (220 * $s)
    $bg = New-Object System.Drawing.Drawing2D.LinearGradientBrush ([System.Drawing.PointF]::new(0, 40 * $s)), ([System.Drawing.PointF]::new(0, 984 * $s)), ([System.Drawing.Color]::FromArgb(255, 44, 46, 52)), ([System.Drawing.Color]::FromArgb(255, 22, 23, 26))
    $g.FillPath($bg, $tile)
    $edge = New-Object System.Drawing.Pen ([System.Drawing.Color]::FromArgb(22, 255, 255, 255)), ([float](4 * $s))
    $g.DrawPath($edge, $tile)
  }

  if (-not $withTile) {
    # Küçük boyut varyantı: zemin yok, simge alanı doldurur (koyu başlık çubuğunda okunurluk).
    $g.TranslateTransform(512 * $s, 512 * $s); $g.ScaleTransform(1.6, 1.6); $g.TranslateTransform(-528 * $s, -512 * $s)
  }

  # Üç oyun kutusu: arkadan öne, her biri biraz daha açık ve daha dik.
  $cases = @(
    @{ X = 318; Y = 268; Angle = -14; Alpha = 90 },
    @{ X = 378; Y = 288; Angle = -6; Alpha = 160 },
    @{ X = 438; Y = 308; Angle = 4; Alpha = 255 }
  )
  foreach ($c in $cases) {
    $state = $g.Save()
    $cx = ($c.X + 150) * $s; $cy = ($c.Y + 230) * $s
    $g.TranslateTransform($cx, $cy); $g.RotateTransform($c.Angle); $g.TranslateTransform(-$cx, -$cy)
    $rect = New-RoundRect ($c.X * $s) ($c.Y * $s) (300 * $s) (460 * $s) (44 * $s)
    $brush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb($c.Alpha, 237, 237, 238))
    $g.FillPath($brush, $rect)
    if ($c.Alpha -eq 255) {
      # Öndeki kutunun sırtı (spine): koyu ince şerit — kutu hissi verir.
      $spine = New-RoundRect (($c.X + 34) * $s) (($c.Y + 40) * $s) (26 * $s) (380 * $s) (13 * $s)
      $g.FillPath((New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 30, 31, 35))), $spine)
    }
    $g.Restore($state)
  }

  $bmp.Save($file, [System.Drawing.Imaging.ImageFormat]::Png)
  $g.Dispose(); $bmp.Dispose()
}

Draw-Logo 1024 (Join-Path $OutDir 'logo-1024.png') $true
# Başlık çubuğu: 20 px gösterim → 1x/2x/3x, zeminsiz varyant
foreach ($k in @(@{ S = 20; N = 'logo-20.png' }, @{ S = 40; N = 'logo-20@2x.png' }, @{ S = 60; N = 'logo-20@3x.png' })) {
  Draw-Logo $k.S (Join-Path $OutDir $k.N) $false
}
Get-ChildItem $OutDir | Select-Object Name, Length
