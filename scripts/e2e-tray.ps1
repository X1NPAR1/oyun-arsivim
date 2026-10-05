# Gerçek release exe ile tepsi / bellek / sağ tık uçtan uca testi.
# Kullanım: powershell -ExecutionPolicy Bypass -File scripts/e2e-tray.ps1 -Phase 03
param([string] $Phase = "03")
$ErrorActionPreference = 'Stop'
$root = Resolve-Path (Join-Path $PSScriptRoot '..')
$exe = Join-Path $root 'src-tauri\target\release\oyun-arsivim.exe'
$capture = Join-Path $root '.claude\skills\design-review\scripts\capture-window.ps1'

Add-Type @'
using System; using System.Runtime.InteropServices;
public static class E2E {
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint x, uint y, uint d, IntPtr e);
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
  [DllImport("dwmapi.dll")] public static extern int DwmGetWindowAttribute(IntPtr h, int a, out RECT r, int s);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  public struct RECT { public int L, T, R, B; }
}
'@
[E2E]::SetProcessDPIAware() | Out-Null

function Tree-MB([int] $rootPid) {
  # Uygulama + WebView2 alt süreçlerinin toplam özel belleği.
  $all = Get-CimInstance Win32_Process
  $ids = @($rootPid); $added = $true
  while ($added) {
    $added = $false
    foreach ($p in $all) { if ($ids -contains $p.ParentProcessId -and -not ($ids -contains $p.ProcessId)) { $ids += $p.ProcessId; $added = $true } }
  }
  $sum = 0
  foreach ($id in $ids) { $pr = Get-Process -Id $id -EA 0; if ($pr) { $sum += $pr.PrivateMemorySize64 } }
  [pscustomobject]@{ Count = $ids.Count; MB = [math]::Round($sum / 1MB) }
}
function Main-Window {
  # Tek örnek eklentisinin gizli "-siw" mesaj penceresi ana pencere sayılmaz.
  Get-Process oyun-arsivim -EA 0 | Where-Object { $_.MainWindowHandle -ne 0 -and $_.MainWindowTitle -notlike '*-siw' } | Select-Object -First 1
}
function Wait-Until([scriptblock] $cond, [int] $sec = 30) {
  $sw = [Diagnostics.Stopwatch]::StartNew()
  while (-not (& $cond)) { if ($sw.Elapsed.TotalSeconds -gt $sec) { throw "zaman aşımı" }; Start-Sleep -Milliseconds 300 }
}

Get-Process oyun-arsivim -EA 0 | Stop-Process -Force
$app = Start-Process $exe -PassThru
Wait-Until { Main-Window }
Start-Sleep 4
$open = Tree-MB $app.Id
"1) Pencere açık: $($open.Count) süreç, $($open.MB) MB"

# Sağ tık menüsü: Kütüphane'ye git, ilk karta sağ tıkla.
& $capture -Phase $Phase -Name e2e-kutuphane -ClickX 54 -ClickY 150 -SettleMs 1500 | Out-Null
$w = Main-Window
$r = New-Object E2E+RECT; [E2E]::DwmGetWindowAttribute($w.MainWindowHandle, 9, [ref]$r, 16) | Out-Null
[E2E]::SetCursorPos($r.L + 210, $r.T + 330) | Out-Null
[E2E]::mouse_event(0x0008, 0, 0, 0, [IntPtr]::Zero); [E2E]::mouse_event(0x0010, 0, 0, 0, [IntPtr]::Zero)
Start-Sleep -Milliseconds 500
& $capture -Phase $Phase -Name e2e-sag-tik | Out-Null
"2) Sağ tık menüsü görüntüsü alındı"
[E2E]::PostMessage($w.MainWindowHandle, 0x0100, [IntPtr]0x1B, [IntPtr]::Zero) | Out-Null # ESC

# Pencereyi kapat (WM_CLOSE) → uygulama tepside kalmalı, WebView2 süreçleri kapanmalı.
[E2E]::PostMessage($w.MainWindowHandle, 0x0010, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null
Wait-Until { -not (Main-Window) } 10
Start-Sleep 3
if ($app.HasExited) { throw "HATA: pencere kapanınca uygulama tamamen kapandı (tepside kalmadı)" }
$tray = Tree-MB $app.Id
"3) Tepside: $($tray.Count) süreç, $($tray.MB) MB (pencere açıkken $($open.MB) MB)"

# İkinci başlatma → tek örnek eklentisi mevcut uygulamada pencereyi yeniden oluşturmalı.
$second = Start-Process $exe -PassThru
Wait-Until { Main-Window } 20
Start-Sleep 2
$second.Refresh()
$main = Main-Window
"4) Tekrar açıldı: pencere PID $($main.Id) (ilk süreç $($app.Id)); ikinci kopya kapandı: $($second.HasExited)"
& $capture -Phase $Phase -Name e2e-tepsiden-acildi -SettleMs 1500 | Out-Null

Stop-Process -Id $app.Id -Force
[pscustomobject]@{ OpenMB = $open.MB; TrayMB = $tray.MB; ReopenedSamePid = ($main.Id -eq $app.Id); SecondExited = $second.HasExited } | Format-List
