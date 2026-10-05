import { invoke } from "@tauri-apps/api/core";

export interface ProcessUsage {
  pid: number;
  name: string;
  memoryBytes: number;
  cpuPercent: number;
}

export interface PerfSnapshot {
  cpuPercent: number;
  cpuCores: number[];
  cpuName: string;
  memoryTotal: number;
  memoryUsed: number;
  memoryAvailable: number;
  gpuPercent: number | null;
  gpuMemoryUsed: number | null;
  topProcesses: ProcessUsage[];
}

export interface TweakState {
  id: string;
  title: string;
  description: string;
  category: string;
  recommended: boolean;
  requiresAdmin: boolean;
  requiresRestart: boolean;
  enabled: boolean | null;
}

export interface CleanReport {
  processes: number;
  freedBytes: number;
  availableBefore: number;
  availableAfter: number;
}

export interface PowerScheme {
  guid: string;
  name: string;
  active: boolean;
}

export interface PowerOverview {
  schemes: PowerScheme[];
  active: PowerScheme | null;
  duplicateCount: number;
  recommended: PowerScheme | null;
}

export interface PerfProfile {
  enabled: boolean;
  highPriority: boolean;
  powerPlan: boolean;
  cleanRam: boolean;
  closeApps: boolean;
  muteNotifications: boolean;
}

export interface AppSuggestion {
  exe: string;
  title: string;
  running: boolean;
}

export interface RestoreReport {
  restored: string[];
  failed: string[];
}

export const perfApi = {
  snapshot: () => invoke<PerfSnapshot>("perf_snapshot"),
  tweaks: () => invoke<TweakState[]>("perf_tweaks"),
  setTweak: (id: string, enabled: boolean) => invoke<TweakState>("perf_set_tweak", { id, enabled }),
  restoreDefaults: () => invoke<RestoreReport>("perf_restore_defaults"),
  cleanRam: () => invoke<CleanReport>("perf_clean_ram"),
  purgeStandby: () => invoke<null>("perf_purge_standby"),
  power: () => invoke<PowerOverview>("power_overview"),
  setPower: (guid: string) => invoke<PowerOverview>("power_set", { guid }),
  profile: (gameId: number) => invoke<PerfProfile>("perf_profile_get", { gameId }),
  setProfile: (gameId: number, profile: PerfProfile) =>
    invoke<PerfProfile>("perf_profile_set", { gameId, profile }),
  recommendedProfile: () => invoke<PerfProfile>("perf_profile_recommended"),
  profiledGames: () => invoke<number[]>("perf_profiled_games"),
  closeSuggestions: () => invoke<AppSuggestion[]>("perf_close_suggestions"),
};

export function formatGB(bytes: number): string {
  return `${(bytes / 1024 ** 3).toLocaleString("tr-TR", { maximumFractionDigits: 1, minimumFractionDigits: 1 })} GB`;
}

export function formatMB(bytes: number): string {
  return `${Math.round(bytes / 1024 ** 2).toLocaleString("tr-TR")} MB`;
}

/** Yük seviyesi: renk tek başına bilgi taşımaz, yanında metin etiketi gösterilir. */
export function loadLevel(percent: number): { level: "ok" | "high" | "critical"; label: string } {
  if (percent >= 95) return { level: "critical", label: "Kritik" };
  if (percent >= 85) return { level: "high", label: "Yüksek" };
  return { level: "ok", label: "Normal" };
}
