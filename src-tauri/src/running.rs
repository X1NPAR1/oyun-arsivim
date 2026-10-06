//! Şu an çalışan oyun(lar) ve yeni başlatılmış, süreci henüz görünmemiş oyun.
//! Aynı anda tek oyun kuralı buradan uygulanır: takipçi her turda günceller, `launch_game` okur.
use std::collections::HashMap;

use serde::Serialize;

/// Başlatılan oyunun süreci bu kadar saniye içinde görünmezse "başlatılıyor" durumu düşer
/// (Steam/Epic gibi başlatıcılar oyunu birkaç saniye sonra açar).
pub const PENDING_TIMEOUT_SECS: i64 = 90;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunningGame {
    pub id: i64,
    pub title: String,
    /// Oturumun başladığı an (unix sn).
    pub since: i64,
    /// Süreç henüz görünmedi; başlatıcı açıyor.
    pub starting: bool,
}

#[derive(Debug, Default)]
pub struct Running {
    /// Oyun → (süreç kimlikleri, ilk görülme anı).
    live: HashMap<i64, (Vec<u32>, i64)>,
    /// Son başlatılan oyun ve başlatma anı.
    pending: Option<(i64, i64)>,
    titles: HashMap<i64, String>,
}

impl Running {
    /// Takipçi turu: çalışan oyunlar ve süreçleri. Durum değiştiyse `true`.
    pub fn update(&mut self, now: i64, live: &HashMap<i64, Vec<u32>>) -> bool {
        let before = self.snapshot(now);
        self.live.retain(|id, _| live.contains_key(id));
        for (id, pids) in live {
            self.live
                .entry(*id)
                .and_modify(|e| e.0.clone_from(pids))
                .or_insert_with(|| (pids.clone(), now));
        }
        if let Some((id, at)) = self.pending {
            if self.live.contains_key(&id) || now - at >= PENDING_TIMEOUT_SECS {
                self.pending = None;
            }
        }
        before != self.snapshot(now)
    }

    pub fn set_titles(&mut self, titles: HashMap<i64, String>) {
        self.titles = titles;
    }

    pub fn mark_launched(&mut self, id: i64, title: &str, now: i64) {
        self.titles.insert(id, title.to_owned());
        self.pending = Some((id, now));
    }

    /// Çalışan (veya başlatılmakta olan) oyunlar, en eski önce.
    pub fn snapshot(&self, now: i64) -> Vec<RunningGame> {
        let title = |id: i64| self.titles.get(&id).cloned().unwrap_or_default();
        let mut out: Vec<RunningGame> = self
            .live
            .iter()
            .map(|(id, (_, since))| RunningGame {
                id: *id,
                title: title(*id),
                since: *since,
                starting: false,
            })
            .collect();
        if let Some((id, at)) = self.pending {
            if !self.live.contains_key(&id) && now - at < PENDING_TIMEOUT_SECS {
                out.push(RunningGame {
                    id,
                    title: title(id),
                    since: at,
                    starting: true,
                });
            }
        }
        out.sort_by_key(|g| (g.since, g.id));
        out
    }

    /// `id` dışında çalışan ilk oyun (tek oyun kuralı).
    pub fn other_than(&self, id: i64, now: i64) -> Option<RunningGame> {
        self.snapshot(now).into_iter().find(|g| g.id != id)
    }

    /// Oyunun süreçleri (kapatmak için). Başlatılıyor durumundaysa boş.
    pub fn pids_of(&self, id: i64) -> Vec<u32> {
        self.live.get(&id).map(|e| e.0.clone()).unwrap_or_default()
    }

    pub fn forget(&mut self, id: i64) {
        self.live.remove(&id);
        if self.pending.is_some_and(|(p, _)| p == id) {
            self.pending = None;
        }
    }
}

/// Oyun süreçlerini sonlandırır. Yalnız takipçinin oyun klasörüyle eşleştirdiği süreçler verilir;
/// sistem süreçleri asla bu listeye girmez. Kapatılan süreç sayısı döner.
pub fn terminate(pids: &[u32]) -> usize {
    let mut sys = sysinfo::System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    pids.iter()
        .filter_map(|pid| sys.process(sysinfo::Pid::from_u32(*pid)))
        .filter(|p| p.kill())
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn live(items: &[(i64, &[u32])]) -> HashMap<i64, Vec<u32>> {
        items.iter().map(|(id, p)| (*id, p.to_vec())).collect()
    }

    #[test]
    fn pending_launch_counts_as_running_until_process_or_timeout() {
        let mut r = Running::default();
        r.mark_launched(1, "A", 100);
        assert_eq!(r.other_than(2, 101).map(|g| g.id), Some(1));
        assert!(r.snapshot(101)[0].starting);
        // Süreç göründü → artık "başlatılıyor" değil.
        assert!(r.update(110, &live(&[(1, &[42])])));
        let s = r.snapshot(110);
        assert_eq!((s.len(), s[0].starting, s[0].since), (1, false, 110));
        assert_eq!(r.pids_of(1), vec![42]);
        // Kapandı.
        assert!(r.update(120, &live(&[])));
        assert!(r.snapshot(120).is_empty());
        // Hiç süreç görünmeyen başlatma zaman aşımında düşer.
        r.mark_launched(3, "C", 200);
        assert!(r.other_than(9, 200 + PENDING_TIMEOUT_SECS - 1).is_some());
        assert!(r.other_than(9, 200 + PENDING_TIMEOUT_SECS).is_none());
    }

    #[test]
    fn same_game_is_not_a_conflict_and_update_reports_changes() {
        let mut r = Running::default();
        assert!(!r.update(1, &live(&[])));
        assert!(r.update(1, &live(&[(5, &[1])])));
        assert!(!r.update(2, &live(&[(5, &[1, 2])])));
        assert!(r.other_than(5, 2).is_none());
        r.forget(5);
        assert!(r.snapshot(3).is_empty());
    }
}
