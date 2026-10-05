# Oyun Arşivim penceresinin ekran görüntüsünü review/faz-XX/ altına kaydeder.
param(
  [Parameter(Mandatory)] [string] $Phase,
  [Parameter(Mandatory)] [string] $Name,
  [switch] $Launch,
  [int] $Burst = 1,
  [int] $WaitSeconds = 4
)
$ErrorActionPreference = 'Stop'
$root = Resolve-Path (Join-Path $PSScriptRoot '..\..\..\..')
$outDir = Join-Path $root "review\faz-$Phase"
New-Item -ItemType Directory -Force $outDir | Out-Null

Add-Type -AssemblyName System.Drawing
Add-Type @'
using System; using System.Runtime.InteropServices;
public static class OaWin {
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("dwmapi.dll")] public static extern int DwmGetWindowAttribute(IntPtr h, int a, out RECT r, int s);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  public struct RECT { public int Left, Top, Right, Bottom; }
}
'@
[OaWin]::SetProcessDPIAware() | Out-Null

if ($Launch) {
  $exe = Join-Path $root 'src-tauri\target\release\oyun-arsivim.exe'
  $proc = Start-Process $exe -PassThru
  Start-Sleep -Seconds $WaitSeconds
} else {
  $proc = Get-Process oyun-arsivim -ErrorAction Stop | Where-Object { $_.MainWindowHandle -ne 0 } | Select-Object -First 1
}
$proc.Refresh()
$h = $proc.MainWindowHandle
[OaWin]::SetForegroundWindow($h) | Out-Null
Start-Sleep -Milliseconds 600

# DWMWA_EXTENDED_FRAME_BOUNDS (9): gölgesiz gerçek pencere sınırları
$r = New-Object OaWin+RECT
[OaWin]::DwmGetWindowAttribute($h, 9, [ref]$r, 16) | Out-Null
$w = $r.Right - $r.Left
$hgt = $r.Bottom - $r.Top
for ($i = 0; $i -lt $Burst; $i++) {
  $bmp = New-Object System.Drawing.Bitmap $w, $hgt
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.CopyFromScreen($r.Left, $r.Top, 0, 0, $bmp.Size)
  $suffix = if ($Burst -gt 1) { "-$i" } else { '' }
  $file = Join-Path $outDir "$Name$suffix.png"
  $bmp.Save($file, [System.Drawing.Imaging.ImageFormat]::Png)
  $g.Dispose(); $bmp.Dispose()
  Write-Output $file
  if ($Burst -gt 1) { Start-Sleep -Milliseconds 80 }
}
Write-Output ("RAM: {0} MB" -f [int]($proc.WorkingSet64 / 1MB))
if ($Launch) { Stop-Process -Id $proc.Id }
