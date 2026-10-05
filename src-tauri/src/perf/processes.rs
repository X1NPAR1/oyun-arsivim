//! Oyun süreçlerine CPU önceliği ve oyun sırasında seçili arka plan uygulamalarını kapatma.
//!
//! Güvenlik: sistem, güvenlik ve Windows kabuğu süreçleri ASLA kapatılmaz (korumalı liste); yalnız
//! kullanıcının listesindeki ve kendi oturumundaki süreçler hedeflenir. Kapatma önce nazikçe
//! (pencereye kapanma isteği) yapılır; zorla sonlandırma yalnız kullanıcı açıkça isterse.
use serde::Serialize;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

/// Asla dokunulmayan süreçler (küçük harf, .exe ile).
pub const PROTECTED: &[&str] = &[
    "system",
    "registry",
    "smss.exe",
    "csrss.exe",
    "wininit.exe",
    "winlogon.exe",
    "services.exe",
    "lsass.exe",
    "lsaiso.exe",
    "svchost.exe",
    "dwm.exe",
    "explorer.exe",
    "fontdrvhost.exe",
    "sihost.exe",
    "ctfmon.exe",
    "audiodg.exe",
    "spoolsv.exe",
    "taskhostw.exe",
    "runtimebroker.exe",
    "searchhost.exe",
    "startmenuexperiencehost.exe",
    "shellexperiencehost.exe",
    "textinputhost.exe",
    "securityhealthservice.exe",
    "securityhealthsystray.exe",
    "msmpeng.exe",
    "nissrv.exe",
    "mpdefendercoreservice.exe",
    "smartscreen.exe",
    "conhost.exe",
    "dllhost.exe",
    "wudfhost.exe",
    "steam.exe",
    "steamwebhelper.exe",
    "epicgameslauncher.exe",
    "oyun-arsivim.exe",
    "msedgewebview2.exe",
];

/// Arayüzde öneri olarak sunulan yaygın arka plan uygulamaları.
pub const SUGGESTIONS: &[(&str, &str)] = &[
    ("OneDrive.exe", "OneDrive"),
    ("Teams.exe", "Microsoft Teams"),
    ("ms-teams.exe", "Microsoft Teams (yeni)"),
    ("Spotify.exe", "Spotify"),
    ("chrome.exe", "Google Chrome"),
    ("msedge.exe", "Microsoft Edge"),
    ("firefox.exe", "Firefox"),
    ("opera.exe", "Opera"),
    ("Skype.exe", "Skype"),
    ("Dropbox.exe", "Dropbox"),
    ("GoogleDriveFS.exe", "Google Drive"),
    ("AdobeCollabSync.exe", "Adobe Acrobat eşitleme"),
    ("Creative Cloud.exe", "Adobe Creative Cloud"),
    ("brave.exe", "Brave"),
    ("Discord.exe", "Discord"),
    ("Telegram.exe", "Telegram"),
    ("WhatsApp.exe", "WhatsApp"),
];

pub fn is_protected(name: &str) -> bool {
    let n = name.to_lowercase();
    PROTECTED.contains(&n.as_str())
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloseReport {
    pub closed: Vec<String>,
    pub still_running: Vec<String>,
    pub skipped_protected: Vec<String>,
}

fn find_pids(sys: &System, name: &str) -> Vec<u32> {
    sys.processes()
        .values()
        .filter(|p| p.name().to_string_lossy().eq_ignore_ascii_case(name))
        .map(|p| p.pid().as_u32())
        .collect()
}

/// Listedeki uygulamaları nazikçe kapatır; `force` ise yanıt vermeyenleri sonlandırır.
pub fn close_apps(names: &[String], force: bool) -> CloseReport {
    let mut report = CloseReport::default();
    let mut sys = System::new();
    sys.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());
    let mut targets = Vec::new();
    for name in names {
        if is_protected(name) {
            report.skipped_protected.push(name.clone());
            continue;
        }
        let pids = find_pids(&sys, name);
        if !pids.is_empty() {
            for pid in &pids {
                platform::request_close(*pid);
            }
            targets.push(name.clone());
        }
    }
    // Uygulamalara kaydetme/kapanma için süre tanı.
    for _ in 0..10 {
        std::thread::sleep(std::time::Duration::from_millis(300));
        sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing(),
        );
        if targets.iter().all(|n| find_pids(&sys, n).is_empty()) {
            break;
        }
    }
    for name in targets {
        let remaining = find_pids(&sys, &name);
        if remaining.is_empty() {
            report.closed.push(name);
        } else if force {
            remaining.iter().for_each(|pid| platform::terminate(*pid));
            report.closed.push(name);
        } else {
            report.still_running.push(name);
        }
    }
    report
}

/// Oyun süreçlerine yüksek öncelik verir. Dönüş: önceliği değişen süreç sayısı.
pub fn boost_priority(pids: &[u32]) -> usize {
    pids.iter()
        .filter(|pid| platform::set_high_priority(**pid))
        .count()
}

/// Kapatma sonrası yeniden başlatmak için uygulamaların exe yolları.
pub fn exe_paths(names: &[String]) -> Vec<String> {
    let mut sys = System::new();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_exe(UpdateKind::OnlyIfNotSet),
    );
    let mut out: Vec<String> = names
        .iter()
        .filter_map(|n| {
            sys.processes()
                .values()
                .find(|p| p.name().to_string_lossy().eq_ignore_ascii_case(n))
                .and_then(|p| p.exe().map(|e| e.to_string_lossy().into_owned()))
        })
        .collect();
    out.dedup();
    out
}

#[cfg(windows)]
mod platform {
    use windows::core::BOOL;
    use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM, WPARAM};
    use windows::Win32::System::Threading::{
        OpenProcess, SetPriorityClass, TerminateProcess, HIGH_PRIORITY_CLASS,
        PROCESS_SET_INFORMATION, PROCESS_TERMINATE,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowThreadProcessId, PostMessageW, WM_CLOSE,
    };

    unsafe extern "system" fn close_windows_of(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let mut pid = 0u32;
        // SAFETY: Windows'un verdiği geçerli pencere tanıtıcısı.
        unsafe {
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            if pid as isize == lparam.0 {
                let _ = PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0));
            }
        }
        BOOL(1)
    }

    /// Süreç pencerelerine kapanma isteği gönderir (uygulama kaydetme fırsatı bulur).
    pub fn request_close(pid: u32) {
        // SAFETY: geri çağırma yalnız bu çağrı süresince kullanılır.
        unsafe {
            let _ = EnumWindows(Some(close_windows_of), LPARAM(pid as isize));
        }
    }

    pub fn terminate(pid: u32) {
        // SAFETY: tanıtıcı kapatılır.
        unsafe {
            if let Ok(h) = OpenProcess(PROCESS_TERMINATE, false, pid) {
                let _ = TerminateProcess(h, 1);
                let _ = CloseHandle(h);
            }
        }
    }

    pub fn set_high_priority(pid: u32) -> bool {
        // SAFETY: tanıtıcı kapatılır. HIGH (gerçek zamanlı değil) yönetici gerektirmez.
        unsafe {
            let Ok(h) = OpenProcess(PROCESS_SET_INFORMATION, false, pid) else {
                return false;
            };
            let ok = SetPriorityClass(h, HIGH_PRIORITY_CLASS).is_ok();
            let _ = CloseHandle(h);
            ok
        }
    }
}

#[cfg(not(windows))]
mod platform {
    pub fn request_close(_pid: u32) {}
    pub fn terminate(_pid: u32) {}
    pub fn set_high_priority(_pid: u32) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_and_security_processes_are_protected() {
        for n in [
            "svchost.exe",
            "LSASS.EXE",
            "explorer.exe",
            "MsMpEng.exe",
            "csrss.exe",
            "oyun-arsivim.exe",
        ] {
            assert!(is_protected(n), "{n}");
        }
        assert!(!is_protected("Spotify.exe"));
        // Önerilerin hiçbiri korumalı değil (aksi hâlde kullanıcıya işe yaramaz öneri sunulur).
        assert!(SUGGESTIONS.iter().all(|(exe, _)| !is_protected(exe)));
    }

    #[test]
    fn protected_names_are_never_closed() {
        let r = close_apps(&["explorer.exe".into(), "svchost.exe".into()], true);
        assert_eq!(r.skipped_protected.len(), 2);
        assert!(r.closed.is_empty());
    }

    #[cfg(windows)]
    #[test]
    fn closes_a_real_test_app_and_boosts_priority() {
        // Kendi başlattığımız bir test süreci (ping) üzerinde öncelik ve kapatma denenir.
        let tmp = tempfile::tempdir().unwrap();
        let exe = tmp.path().join("oa_kapatma_testi.exe");
        std::fs::copy(r"C:\Windows\System32\PING.EXE", &exe).unwrap();
        let mut child = std::process::Command::new(&exe)
            .args(["-n", "30", "127.0.0.1"])
            .stdout(std::process::Stdio::null())
            .spawn()
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(400));
        assert_eq!(boost_priority(&[child.id()]), 1);
        // Pencere yok → nazik kapatma işe yaramaz, zorla kapatılır.
        let r = close_apps(&["oa_kapatma_testi.exe".into()], true);
        assert_eq!(r.closed, vec!["oa_kapatma_testi.exe".to_string()]);
        let _ = child.wait();
    }
}
