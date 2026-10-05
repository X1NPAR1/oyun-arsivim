# Uçtan uca: oyun başlayınca performans profili (güç planı) uygulanıyor, oyun kapanınca geri alınıyor mu?
# Ön koşul: "Sahin Oyunu" test oyunu kütüphanede ve profili açık (bkz. boost::real_system).
param([int] $PlaySeconds = 40)
$ErrorActionPreference = 'Stop'
$exe = 'D:\OyunArsivim\app\src-tauri\target\release\oyun-arsivim.exe'
$game = "D:\OyunArsivim\test-oyun\Şahin'in Oyunu\Sahin_Oyunu.exe"
function ActivePlan { (powercfg /getactivescheme) -replace '.*GUID:\s*([0-9a-f-]+).*', '$1' }

Get-Process oyun-arsivim, Sahin_Oyunu -EA 0 | Stop-Process -Force
$before = ActivePlan
"Başlangıç planı: $before"
$app = Start-Process $exe -ArgumentList '--hidden' -PassThru   # tepside, pencere yok
Start-Sleep 6
$g = Start-Process $game -ArgumentList '-n', '400', '127.0.0.1' -WindowStyle Hidden -PassThru
Start-Sleep 15
$during = ActivePlan
$prio = (Get-Process -Id $g.Id).PriorityClass
"Oyun çalışırken plan: $during · oyun süreci önceliği: $prio"
Start-Sleep ($PlaySeconds - 15)
Stop-Process -Id $g.Id -Force
Start-Sleep 20
$after = ActivePlan
"Oyun kapandıktan sonra plan: $after"
Stop-Process -Id $app.Id -Force
[pscustomobject]@{
  PlanDegisti   = ($during -ne $before)
  OncelikYuksek = ($prio -eq 'High')
  GeriAlindi    = ($after -eq $before)
} | Format-List
