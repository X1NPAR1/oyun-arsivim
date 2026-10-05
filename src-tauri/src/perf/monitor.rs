//! Canlı sistem ölçümü: CPU (toplam + çekirdek), RAM, GPU kullanımı ve video belleği, en çok bellek
//! kullanan süreçler. Yalnız Performans sayfası açıkken (arayüz sorduğunda) ölçülür.
//!
//! GPU: Görev Yöneticisi'nin kullandığı PDH sayaçları — `\GPU Engine(*engtype_3D)\Utilization Percentage`
//! (süreç başına 3D motor kullanımlarının toplamı) ve `\GPU Adapter Memory(*)\Dedicated Usage`.
use std::sync::Mutex;

use serde::Serialize;
use sysinfo::{MemoryRefreshKind, ProcessRefreshKind, ProcessesToUpdate, System};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessUsage {
    pub pid: u32,
    pub name: String,
    pub memory_bytes: u64,
    pub cpu_percent: f32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PerfSnapshot {
    pub cpu_percent: f32,
    pub cpu_cores: Vec<f32>,
    pub cpu_name: String,
    pub memory_total: u64,
    pub memory_used: u64,
    pub memory_available: u64,
    /// GPU 3D motor kullanımı (%); sayaç yoksa `None`.
    pub gpu_percent: Option<f64>,
    pub gpu_memory_used: Option<u64>,
    pub top_processes: Vec<ProcessUsage>,
}

pub struct PerfMonitor {
    sys: Mutex<System>,
    gpu: Mutex<Option<gpu::GpuCounters>>,
}

impl Default for PerfMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl PerfMonitor {
    pub fn new() -> Self {
        let mut sys = System::new();
        // CPU yüzdesi iki ölçüm arasındaki farktan hesaplanır; ilk ölçümü baştan al.
        sys.refresh_cpu_all();
        Self {
            sys: Mutex::new(sys),
            gpu: Mutex::new(gpu::GpuCounters::open()),
        }
    }

    pub fn snapshot(&self, top: usize) -> PerfSnapshot {
        let mut sys = self
            .sys
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        sys.refresh_cpu_all();
        sys.refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
        sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_memory().with_cpu(),
        );
        let cores: Vec<f32> = sys.cpus().iter().map(sysinfo::Cpu::cpu_usage).collect();
        let core_count = u16::try_from(cores.len().max(1)).unwrap_or(1);
        let mut procs: Vec<ProcessUsage> = sys
            .processes()
            .values()
            .map(|p| ProcessUsage {
                pid: p.pid().as_u32(),
                name: p.name().to_string_lossy().into_owned(),
                memory_bytes: p.memory(),
                // sysinfo süreç CPU'sunu çekirdek başına verir (8 çekirdekte en fazla %800).
                cpu_percent: p.cpu_usage() / f32::from(core_count),
            })
            .collect();
        procs.sort_by_key(|p| std::cmp::Reverse(p.memory_bytes));
        procs.truncate(top);
        let (gpu_percent, gpu_memory_used) = self
            .gpu
            .lock()
            .ok()
            .and_then(|mut g| g.as_mut().map(gpu::GpuCounters::sample))
            .unwrap_or((None, None));
        PerfSnapshot {
            cpu_percent: sys.global_cpu_usage(),
            cpu_name: sys
                .cpus()
                .first()
                .map(|c| c.brand().trim().to_owned())
                .unwrap_or_default(),
            cpu_cores: cores,
            memory_total: sys.total_memory(),
            memory_used: sys.used_memory(),
            memory_available: sys.available_memory(),
            gpu_percent,
            gpu_memory_used,
            top_processes: procs,
        }
    }
}

#[cfg(windows)]
mod gpu {
    use windows::core::PCWSTR;
    use windows::Win32::System::Performance::{
        PdhAddEnglishCounterW, PdhCloseQuery, PdhCollectQueryData, PdhGetFormattedCounterArrayW,
        PdhOpenQueryW, PDH_FMT_COUNTERVALUE_ITEM_W, PDH_FMT_DOUBLE, PDH_FMT_LARGE, PDH_HCOUNTER,
        PDH_HQUERY, PDH_MORE_DATA,
    };

    pub struct GpuCounters {
        query: PDH_HQUERY,
        util: PDH_HCOUNTER,
        mem: PDH_HCOUNTER,
    }

    // PDH tanıtıcıları iş parçacıkları arasında taşınabilir; erişim Mutex ile korunur.
    unsafe impl Send for GpuCounters {}

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    impl GpuCounters {
        pub fn open() -> Option<Self> {
            let mut query = PDH_HQUERY::default();
            let mut util = PDH_HCOUNTER::default();
            let mut mem = PDH_HCOUNTER::default();
            let util_path = wide(r"\GPU Engine(*engtype_3D)\Utilization Percentage");
            let mem_path = wide(r"\GPU Adapter Memory(*)\Dedicated Usage");
            // SAFETY: çıkış parametreleri geçerli; yollar sıfır sonlu. Hata durumunda sorgu kapatılır.
            unsafe {
                if PdhOpenQueryW(PCWSTR::null(), 0, &mut query) != 0 {
                    return None;
                }
                let ok = PdhAddEnglishCounterW(query, PCWSTR(util_path.as_ptr()), 0, &mut util)
                    == 0
                    && PdhAddEnglishCounterW(query, PCWSTR(mem_path.as_ptr()), 0, &mut mem) == 0;
                if !ok {
                    let _ = PdhCloseQuery(query);
                    return None;
                }
                // Oran sayaçları iki toplama ister; ilkini şimdi al.
                let _ = PdhCollectQueryData(query);
            }
            Some(Self { query, util, mem })
        }

        /// Sayaç örneklerinin toplamı. `large` true ise tamsayı (bayt), değilse ondalık (yüzde).
        fn sum(counter: PDH_HCOUNTER, large: bool) -> Option<f64> {
            let format = if large { PDH_FMT_LARGE } else { PDH_FMT_DOUBLE };
            let mut size = 0u32;
            let mut count = 0u32;
            // SAFETY: önce gereken boyut sorulur, sonra hizalı arabellek ayrılır.
            unsafe {
                let rc = PdhGetFormattedCounterArrayW(counter, format, &mut size, &mut count, None);
                if rc != PDH_MORE_DATA || size == 0 {
                    return None;
                }
                let item = std::mem::size_of::<PDH_FMT_COUNTERVALUE_ITEM_W>();
                let mut buf: Vec<PDH_FMT_COUNTERVALUE_ITEM_W> =
                    Vec::with_capacity((size as usize).div_ceil(item));
                let rc = PdhGetFormattedCounterArrayW(
                    counter,
                    format,
                    &mut size,
                    &mut count,
                    Some(buf.as_mut_ptr()),
                );
                if rc != 0 {
                    return None;
                }
                buf.set_len(count as usize);
                let total = buf
                    .iter()
                    .filter(|i| i.FmtValue.CStatus == 0)
                    .map(|i| {
                        if large {
                            i.FmtValue.Anonymous.largeValue as f64
                        } else {
                            i.FmtValue.Anonymous.doubleValue
                        }
                    })
                    .sum();
                Some(total)
            }
        }

        pub fn sample(&mut self) -> (Option<f64>, Option<u64>) {
            // SAFETY: sorgu geçerli.
            if unsafe { PdhCollectQueryData(self.query) } != 0 {
                return (None, None);
            }
            let util = Self::sum(self.util, false).map(|v| v.clamp(0.0, 100.0));
            let mem = Self::sum(self.mem, true).map(|v| v.max(0.0) as u64);
            (util, mem)
        }
    }

    impl Drop for GpuCounters {
        fn drop(&mut self) {
            // SAFETY: open() içinde açılan sorgu.
            unsafe {
                let _ = PdhCloseQuery(self.query);
            }
        }
    }
}

#[cfg(not(windows))]
mod gpu {
    pub struct GpuCounters;
    impl GpuCounters {
        pub fn open() -> Option<Self> {
            None
        }
        pub fn sample(&mut self) -> (Option<f64>, Option<u64>) {
            (None, None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_reports_sane_values_on_real_system() {
        let m = PerfMonitor::new();
        std::thread::sleep(std::time::Duration::from_millis(250));
        let s = m.snapshot(5);
        assert!(s.memory_total > 0);
        assert!(s.memory_used <= s.memory_total);
        assert!((0.0..=100.0).contains(&s.cpu_percent));
        assert!(!s.cpu_cores.is_empty());
        assert!(s.top_processes.len() <= 5);
        assert!(s
            .top_processes
            .windows(2)
            .all(|w| w[0].memory_bytes >= w[1].memory_bytes));
        if let Some(g) = s.gpu_percent {
            assert!((0.0..=100.0).contains(&g));
        }
    }
}

#[cfg(test)]
mod real_system {
    use super::*;

    #[test]
    #[ignore = "gerçek donanım değerlerini yazdırır"]
    fn print_snapshot() {
        let m = PerfMonitor::new();
        for _ in 0..2 {
            std::thread::sleep(std::time::Duration::from_millis(600));
            let s = m.snapshot(3);
            println!(
                "CPU {:.1}% ({} çekirdek, {}) RAM {}/{} MB GPU {:?}% VRAM {:?} MB",
                s.cpu_percent,
                s.cpu_cores.len(),
                s.cpu_name,
                s.memory_used / 1_048_576,
                s.memory_total / 1_048_576,
                s.gpu_percent.map(|g| (g * 10.0).round() / 10.0),
                s.gpu_memory_used.map(|b| b / 1_048_576)
            );
        }
    }
}
