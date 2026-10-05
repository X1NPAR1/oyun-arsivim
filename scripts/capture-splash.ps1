# Açılış penceresini (splash) yakalar: exe'yi başlatır, splash'i bulur, kısa aralıklarla kare alır.
# PrintWindow kullanılır: yalnız bu pencerenin içeriği çizilir; üstteki başka pencereler asla yakalanmaz
# ve odak kilidinden etkilenmez.
param(
  [string] $Exe = 'D:\OyunArsivim\app\src-tauri\target\release\oyun-arsivim.exe',
  [string] $Name = 'splash',
  [int] $Frames = 14,
  [int] $IntervalMs = 120
)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System; using System.Runtime.InteropServices;
public static class Sp {
  public delegate bool EnumProc(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc f, IntPtr l);
  [DllImport("user32.dll")] public static extern int GetWindowThreadProcessId(IntPtr h, out int pid);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool IsWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr hdc, uint flags);
  [DllImport("dwmapi.dll")] public static extern int DwmGetWindowAttribute(IntPtr h, int a, out RECT r, int s);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  public struct RECT { public int L, T, R, B; }
}
'@
[Sp]::SetProcessDPIAware() | Out-Null
$out = 'D:\OyunArsivim\app\review\faz-06'
New-Item -ItemType Directory -Force $out | Out-Null
Get-ChildItem $out -Filter "$Name-*.png" -EA 0 | Remove-Item
Get-Process oyun-arsivim -EA 0 | Stop-Process -Force
$p = Start-Process $Exe -PassThru
$script:found = [IntPtr]::Zero
$sw = [Diagnostics.Stopwatch]::StartNew()
while ($script:found -eq [IntPtr]::Zero -and $sw.ElapsedMilliseconds -lt 8000) {
  [Sp]::EnumWindows({
      param($h, $l)
      $wpid = 0
      [Sp]::GetWindowThreadProcessId($h, [ref]$wpid) | Out-Null
      if ($wpid -eq $p.Id -and [Sp]::IsWindowVisible($h)) {
        $r = New-Object Sp+RECT
        [Sp]::DwmGetWindowAttribute($h, 9, [ref]$r, 16) | Out-Null
        $w = $r.R - $r.L
        if ($w -gt 100 -and $w -lt 600) { $script:found = $h; return $false }
      }
      return $true
    }, [IntPtr]::Zero) | Out-Null
  Start-Sleep -Milliseconds 20
}
if ($script:found -eq [IntPtr]::Zero) { throw "splash penceresi bulunamadı" }
"splash göründü: $($sw.ElapsedMilliseconds) ms"
$saved = 0
for ($i = 0; $i -lt $Frames; $i++) {
  if (-not [Sp]::IsWindow($script:found)) { "splash kapandı: $($sw.ElapsedMilliseconds) ms"; break }
  $r = New-Object Sp+RECT
  [Sp]::DwmGetWindowAttribute($script:found, 9, [ref]$r, 16) | Out-Null
  $w = $r.R - $r.L; $h = $r.B - $r.T
  if ($w -le 0 -or $h -le 0) { break }
  $bmp = New-Object System.Drawing.Bitmap $w, $h
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.Clear([System.Drawing.Color]::FromArgb(255, 40, 40, 44))
  $hdc = $g.GetHdc()
  [Sp]::PrintWindow($script:found, $hdc, 2) | Out-Null
  $g.ReleaseHdc($hdc)
  $bmp.Save("$out\$Name-$i.png")
  $g.Dispose(); $bmp.Dispose(); $saved++
  Start-Sleep -Milliseconds $IntervalMs
}
"kaydedilen kare: $saved · toplam: $($sw.ElapsedMilliseconds) ms"
