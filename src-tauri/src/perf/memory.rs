//! RAM temizliği.
//!
//! 1. Çalışma kümesi kırpma (`EmptyWorkingSet`): erişilebilen süreçlerin fiziksel bellekteki
//!    kullanılmayan sayfaları serbest bırakılır. Yönetici gerektirmez; veri kaybı olmaz (sayfalar
//!    gerektiğinde geri yüklenir).
//! 2. Bekleme listesi temizliği (yönetici): Windows'un önbellek olarak tuttuğu "bekleme" sayfalarını
//!    boşaltır (`NtSetSystemInformation(SystemMemoryListInformation, MemoryPurgeStandbyList)`).
use serde::Serialize;
use sysinfo::{MemoryRefreshKind, ProcessRefreshKind, ProcessesToUpdate, System};

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanReport {
    pub processes: u32,
    /// Kırpılan süreçlerin çalışma kümesi toplamındaki azalma.
    pub freed_bytes: u64,
    pub available_before: u64,
    pub available_after: u64,
}

fn working_sets(sys: &mut System) -> Vec<(u32, u64)> {
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_memory(),
    );
    sys.processes()
        .values()
        .map(|p| (p.pid().as_u32(), p.memory()))
        .collect()
}

/// Erişilebilen tüm süreçlerin çalışma kümesini kırpar.
pub fn trim_working_sets() -> CleanReport {
    let mut sys = System::new();
    sys.refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
    let available_before = sys.available_memory();
    let before = working_sets(&mut sys);
    let mut trimmed = Vec::new();
    for (pid, _) in &before {
        if platform::empty_working_set(*pid) {
            trimmed.push(*pid);
        }
    }
    // Windows istatistikleri kısa gecikmeyle güncellenir.
    std::thread::sleep(std::time::Duration::from_millis(400));
    let after: std::collections::HashMap<u32, u64> = working_sets(&mut sys).into_iter().collect();
    let freed = before
        .iter()
        .filter(|(pid, _)| trimmed.contains(pid))
        .map(|(pid, b)| b.saturating_sub(after.get(pid).copied().unwrap_or(*b)))
        .sum();
    sys.refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
    CleanReport {
        processes: u32::try_from(trimmed.len()).unwrap_or(u32::MAX),
        freed_bytes: freed,
        available_before,
        available_after: sys.available_memory(),
    }
}

/// Otomatik temizlik kararı: kullanılan bellek eşik yüzdesini aştı mı?
pub fn should_auto_clean(total: u64, available: u64, threshold_percent: u8) -> bool {
    if total == 0 {
        return false;
    }
    let used_percent = (total.saturating_sub(available)) * 100 / total;
    used_percent >= u64::from(threshold_percent)
}

pub fn purge_standby_list() -> bool {
    platform::purge_standby_list()
}

#[cfg(windows)]
mod platform {
    use windows::Win32::Foundation::{CloseHandle, HANDLE, LUID};
    use windows::Win32::Security::{
        AdjustTokenPrivileges, LookupPrivilegeValueW, SE_PRIVILEGE_ENABLED,
        TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, TOKEN_QUERY,
    };
    use windows::Win32::System::ProcessStatus::EmptyWorkingSet;
    use windows::Win32::System::Threading::{
        GetCurrentProcess, OpenProcess, OpenProcessToken, PROCESS_QUERY_LIMITED_INFORMATION,
        PROCESS_SET_QUOTA,
    };

    pub fn empty_working_set(pid: u32) -> bool {
        if pid == 0 || pid == 4 {
            return false; // Sistem boşta süreci / System
        }
        // SAFETY: tanıtıcı açılırsa kapatılır; erişim yoksa (korumalı süreç) sessizce atlanır.
        unsafe {
            let Ok(h) = OpenProcess(
                PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SET_QUOTA,
                false,
                pid,
            ) else {
                return false;
            };
            let ok = EmptyWorkingSet(h).is_ok();
            let _ = CloseHandle(h);
            ok
        }
    }

    fn enable_privilege(name: windows::core::PCWSTR) -> bool {
        // SAFETY: kendi süreç belirtecimiz; tanıtıcı kapatılır.
        unsafe {
            let mut token = HANDLE::default();
            if OpenProcessToken(
                GetCurrentProcess(),
                TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
                &mut token,
            )
            .is_err()
            {
                return false;
            }
            let mut luid = LUID::default();
            let ok = LookupPrivilegeValueW(None, name, &mut luid).is_ok() && {
                let mut tp = TOKEN_PRIVILEGES {
                    PrivilegeCount: 1,
                    ..Default::default()
                };
                tp.Privileges[0].Luid = luid;
                tp.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;
                AdjustTokenPrivileges(token, false, Some(&tp), 0, None, None).is_ok()
                    && windows::Win32::Foundation::GetLastError().is_ok()
            };
            let _ = CloseHandle(token);
            ok
        }
    }

    #[link(name = "ntdll")]
    unsafe extern "system" {
        fn NtSetSystemInformation(class: i32, info: *const core::ffi::c_void, len: u32) -> i32;
    }

    pub fn purge_standby_list() -> bool {
        // SystemMemoryListInformation = 80, MemoryPurgeStandbyList = 4 (belgelenmiş ntdll arayüzü;
        // RAMMap/ISLC ile aynı çağrı).
        const SYSTEM_MEMORY_LIST_INFORMATION: i32 = 80;
        const MEMORY_PURGE_STANDBY_LIST: u32 = 4;
        if !enable_privilege(windows::core::w!("SeProfileSingleProcessPrivilege")) {
            return false;
        }
        let command = MEMORY_PURGE_STANDBY_LIST;
        // SAFETY: 4 baytlık komut değeri; sınıf numarası ntdll sözleşmesine uygun.
        let status = unsafe {
            NtSetSystemInformation(
                SYSTEM_MEMORY_LIST_INFORMATION,
                (&raw const command).cast(),
                u32::try_from(std::mem::size_of::<u32>()).unwrap_or(4),
            )
        };
        status >= 0 // NTSTATUS: negatif değerler hata
    }
}

#[cfg(not(windows))]
mod platform {
    pub fn empty_working_set(_pid: u32) -> bool {
        false
    }
    pub fn purge_standby_list() -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_clean_threshold() {
        let gb = 1024 * 1024 * 1024;
        assert!(should_auto_clean(16 * gb, 2 * gb, 85)); // %87,5 kullanım
        assert!(!should_auto_clean(16 * gb, 4 * gb, 85)); // %75
        assert!(!should_auto_clean(0, 0, 85));
    }

    #[cfg(windows)]
    #[test]
    fn trims_working_sets_on_real_system_without_admin() {
        let r = trim_working_sets();
        assert!(
            r.processes > 0,
            "en az kendi kullanıcı süreçlerimiz kırpılabilmeli"
        );
        assert!(r.available_after > 0);
    }

    #[cfg(windows)]
    #[test]
    fn standby_purge_needs_admin() {
        // Yönetici olmadan ayrıcalık açılamaz → false (çökmeden).
        let _ = purge_standby_list();
    }
}
