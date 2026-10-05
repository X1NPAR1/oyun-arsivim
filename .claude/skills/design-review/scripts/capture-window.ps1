# Oyun Arşivim penceresinin ekran görüntüsünü review/faz-XX/ altına kaydeder.
# -ClickX/-ClickY (pencereye göre piksel) verilirse önce oraya tıklar; -Burst ile geçiş kareleri alınır.
param(
  [Parameter(Mandatory)] [string] $Phase,
  [Parameter(Mandatory)] [string] $Name,
  [switch] $Launch,
  [int] $Burst = 1,
  [int] $IntervalMs = 80,
  [int] $WaitSeconds = 4,
  [int] $ClickX = -1,
  [int] $ClickY = -1,
  [int] $SettleMs = 0
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
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("dwmapi.dll")] public static extern int DwmGetWindowAttribute(IntPtr h, int a, out RECT r, int s);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint x, uint y, uint d, IntPtr e);
  public struct RECT { public int Left, Top, Right, Bottom; }
}
'@
[OaWin]::SetProcessDPIAware() | Out-Null

if ($Launch) {
  $exe = Join-Path $root 'src-tauri\target\release\oyun-arsivim.exe'
  $proc = Start-Process $exe -PassThru
  Start-Sleep -Seconds $WaitSeconds
} else {
  # Tek örnek eklentisinin gizli "-siw" penceresi ana pencere değildir.
  $proc = Get-Process oyun-arsivim -ErrorAction Stop | Where-Object { $_.MainWindowHandle -ne 0 -and $_.MainWindowTitle -notlike '*-siw' } | Select-Object -First 1
  if (-not $proc) { throw "Oyun Arşivim penceresi açık değil" }
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

if ($ClickX -ge 0 -and $ClickY -ge 0) {
  [OaWin]::SetCursorPos($r.Left + $ClickX, $r.Top + $ClickY) | Out-Null
  [OaWin]::mouse_event(0x0002, 0, 0, 0, [IntPtr]::Zero) # sol tuş bas
  [OaWin]::mouse_event(0x0004, 0, 0, 0, [IntPtr]::Zero) # bırak
  # İmleci içerikten uzaklaştır ki hover durumu kareleri bozmasın.
  [OaWin]::SetCursorPos($r.Right - 40, $r.Bottom - 40) | Out-Null
}
if ($SettleMs -gt 0) { Start-Sleep -Milliseconds $SettleMs }

for ($i = 0; $i -lt $Burst; $i++) {
  $bmp = New-Object System.Drawing.Bitmap $w, $hgt
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.CopyFromScreen($r.Left, $r.Top, 0, 0, $bmp.Size)
  $suffix = if ($Burst -gt 1) { "-$i" } else { '' }
  $file = Join-Path $outDir "$Name$suffix.png"
  $bmp.Save($file, [System.Drawing.Imaging.ImageFormat]::Png)
  $g.Dispose(); $bmp.Dispose()
  Write-Output $file
  if ($Burst -gt 1) { Start-Sleep -Milliseconds $IntervalMs }
}
Write-Output ("RAM: {0} MB" -f [int]($proc.WorkingSet64 / 1MB))
if ($Launch) { Stop-Process -Id $proc.Id }
