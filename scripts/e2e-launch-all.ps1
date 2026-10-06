# Kütüphanedeki tüm kurulu oyunları uygulamanın kullandığı yolla başlatır, oyun sürecinin
# kurulum klasöründen doğduğunu doğrular ve oyunu hemen kapatır.
# Arkada oyun kalmaması için: her oyundan sonra geç açılanlar (başlatıcı → oyun) da izlenip kapatılır;
# test bitince testin açtığı başlatıcılar kapatılır. Testten ÖNCE açık olan hiçbir sürece dokunulmaz
# (Steam vb.); hizmetler ve sistem süreçleri listeye hiç girmez.
# Kullanım: powershell -File scripts/e2e-launch-all.ps1 -GamesJson games.json -Out results.json
param(
  [Parameter(Mandatory)] [string] $GamesJson,
  [Parameter(Mandatory)] [string] $Out,
  [int] $TimeoutSec = 150,
  [int] $PlaySec = 6,
  [int] $QuietSec = 25
)
$ErrorActionPreference = 'Continue'
# PS 5.1 diziyi tek nesne olarak verir; ForEach ile açılır.
$games = @(Get-Content -Raw -Encoding UTF8 $GamesJson | ConvertFrom-Json | ForEach-Object { $_ })
$baseline = @{}
Get-Process | ForEach-Object { $baseline[$_.Id] = $true }

# Testin açabileceği başlatıcıların klasörleri (yalnız test sırasında doğan süreçleri kapatılır).
$pf86 = [Environment]::GetFolderPath('ProgramFilesX86')
$launcherDirs = @(
  'D:\Launcher\',
  ($env:ProgramFiles + '\Rockstar Games\'),
  ($env:ProgramFiles + '\Electronic Arts\'),
  ($env:ProgramFiles + '\Epic Games\'),
  ($pf86 + '\Epic Games\'),
  ($env:LOCALAPPDATA + '\Programs\Paradox Interactive\'),
  'F:\Riot Games\Riot Client\',
  ($env:LOCALAPPDATA + '\Roblox\'),
  ($pf86 + '\Ubisoft\'),
  ($pf86 + '\Battle.net\')
)

function Test-Under($path, $dirs) {
  foreach ($d in $dirs) { if ($path.StartsWith($d, [StringComparison]::OrdinalIgnoreCase)) { return $true } }
  return $false
}

function Get-Dirs($game) {
  $dir = $game.install_dir.TrimEnd('\') + '\'
  $i = $dir.IndexOf('\WindowsApps\')
  if ($i -ge 0) {
    # Paket sürümü değişebilir: paket adına göre eşle (ör. Microsoft.MinecraftUWP_).
    return @($dir.Substring(0, $i + 13) + (Split-Path $game.install_dir -Leaf).Split('_')[0] + '_')
  }
  return @($dir)
}

function Get-GameProcs($dirs) {
  Get-Process -ErrorAction SilentlyContinue | Where-Object { $_.Path -and (Test-Under $_.Path $dirs) }
}

$allGameDirs = @($games | ForEach-Object { Get-Dirs $_ })

function Close-Procs($dirs, [switch] $OnlyNew) {
  for ($i = 0; $i -lt 10; $i++) {
    $left = @(Get-GameProcs $dirs | Where-Object { -not $OnlyNew -or -not $baseline.ContainsKey($_.Id) })
    if ($left.Count -eq 0) { return }
    $left | Stop-Process -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 2
  }
}

# Geç açılan oyunları da yakalar: QuietSec boyunca hiçbir oyun klasöründe süreç görülmeyene dek kapatır.
function Wait-Quiet {
  $quietSince = Get-Date
  $deadline = (Get-Date).AddSeconds(180)
  while (((Get-Date) - $quietSince).TotalSeconds -lt $QuietSec -and (Get-Date) -lt $deadline) {
    Start-Sleep -Seconds 2
    $late = @(Get-GameProcs $allGameDirs)
    if ($late.Count -gt 0) {
      Close-Procs $allGameDirs
      $quietSince = Get-Date
    }
  }
}

$results = @()
foreach ($g in $games) {
  $dirs = Get-Dirs $g
  $t0 = Get-Date
  try {
    if ($g.launch_uri) { Start-Process $g.launch_uri }
    elseif ($g.executable) {
      if ($g.launch_args) { Start-Process -FilePath $g.executable -ArgumentList $g.launch_args }
      else { Start-Process -FilePath $g.executable -WorkingDirectory $g.install_dir }
    }
  } catch {
    $results += [pscustomobject]@{ id = $g.id; title = $g.title; ok = $false; seconds = 0; exe = ''; note = ('başlatma hatası: ' + $_.Exception.Message) }
    continue
  }
  $found = @()
  while (((Get-Date) - $t0).TotalSeconds -lt $TimeoutSec) {
    Start-Sleep -Milliseconds 1500
    $found = @(Get-GameProcs $dirs)
    if ($found.Count -gt 0) { break }
  }
  $secs = [int]((Get-Date) - $t0).TotalSeconds
  if ($found.Count -gt 0) {
    Start-Sleep -Seconds $PlaySec
    $names = (@(Get-GameProcs $dirs) | Select-Object -ExpandProperty ProcessName -Unique) -join ','
    $results += [pscustomobject]@{ id = $g.id; title = $g.title; ok = $true; seconds = $secs; exe = $names; note = '' }
  } else {
    $results += [pscustomobject]@{ id = $g.id; title = $g.title; ok = $false; seconds = $secs; exe = ''; note = 'süre doldu: oyun süreci görünmedi' }
  }
  Close-Procs $dirs
  Wait-Quiet
  $results | ConvertTo-Json | Set-Content -Encoding UTF8 $Out
}
# Testin açtığı başlatıcılar (Rockstar, EA, Epic, Riot…) kapatılır; önceden açık olanlar kalır.
Close-Procs $launcherDirs -OnlyNew
Wait-Quiet
$remaining = @(Get-GameProcs $allGameDirs).Count + @(Get-GameProcs $launcherDirs | Where-Object { -not $baseline.ContainsKey($_.Id) }).Count
$results | ConvertTo-Json | Set-Content -Encoding UTF8 $Out
$results | Format-Table -AutoSize | Out-String -Width 200
"Arkada kalan test süreci: $remaining"
