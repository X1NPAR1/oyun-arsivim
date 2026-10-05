# Gerçek release exe ile takipçi uçtan uca testi: diyalogdan elle oyun ekle → oyunu çalıştır →
# oturum ve süre kaydını, istatistik sayfasını doğrula. Kullanıcının oyunlarına dokunmaz.
param([string] $Phase = "04", [int] $PlaySeconds = 75)
$ErrorActionPreference = 'Stop'
$root = Resolve-Path (Join-Path $PSScriptRoot '..')
$exe = Join-Path $root 'src-tauri\target\release\oyun-arsivim.exe'
$capture = Join-Path $root '.claude\skills\design-review\scripts\capture-window.ps1'
$game = "D:\OyunArsivim\test-oyun\Şahin'in Oyunu\Sahin_Oyunu.exe"

Add-Type @'
using System; using System.Runtime.InteropServices;
public static class T {
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint x, uint y, uint d, IntPtr e);
  [DllImport("dwmapi.dll")] public static extern int DwmGetWindowAttribute(IntPtr h, int a, out RECT r, int s);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  public struct RECT { public int L, T, R, B; }
}
'@
[T]::SetProcessDPIAware() | Out-Null
Add-Type -AssemblyName System.Windows.Forms
function Main-Window { Get-Process oyun-arsivim -EA 0 | Where-Object { $_.MainWindowHandle -ne 0 -and $_.MainWindowTitle -notlike '*-siw' } | Select-Object -First 1 }
function Click([int] $x, [int] $y) {
  $w = Main-Window
  $r = New-Object T+RECT; [T]::DwmGetWindowAttribute($w.MainWindowHandle, 9, [ref]$r, 16) | Out-Null
  [T]::SetForegroundWindow($w.MainWindowHandle) | Out-Null
  [T]::SetCursorPos($r.L + $x, $r.T + $y) | Out-Null
  [T]::mouse_event(2, 0, 0, 0, [IntPtr]::Zero); [T]::mouse_event(4, 0, 0, 0, [IntPtr]::Zero)
}

Get-Process oyun-arsivim, Sahin_Oyunu -EA 0 | Stop-Process -Force
$app = Start-Process $exe -PassThru
while (-not (Main-Window)) { Start-Sleep -Milliseconds 300 }
Start-Sleep 4

# 1) Kütüphane → "Oyun ekle" → Windows dosya diyaloğuna yolu yaz.
Click 54 150; Start-Sleep 1
Click 1188 126; Start-Sleep 2
[System.Windows.Forms.SendKeys]::SendWait($game.Replace('(', '{(}').Replace(')', '{)}'))
Start-Sleep -Milliseconds 400
[System.Windows.Forms.SendKeys]::SendWait('{ENTER}')
Start-Sleep 3
& $capture -Phase $Phase -Name e2e-elle-eklendi | Out-Null
"1) Elle ekleme diyaloğu tamamlandı"

# 2) Oyunu başlat; takipçi 5-10 sn içinde yakalamalı.
$g = Start-Process $game -ArgumentList "-n", "400", "127.0.0.1" -WindowStyle Hidden -PassThru
Start-Sleep $PlaySeconds
Stop-Process -Id $g.Id -Force -EA SilentlyContinue
"2) Test oyunu $PlaySeconds sn çalıştı ve kapatıldı"
Start-Sleep 20   # kesinti toleransı (2 ölçüm) + oturum kapanışı

# 3) İstatistikler sayfası
Click 54 215; Start-Sleep 3
& $capture -Phase $Phase -Name e2e-istatistik | Out-Null
"3) İstatistik ekran görüntüsü alındı"
Stop-Process -Id $app.Id -Force
