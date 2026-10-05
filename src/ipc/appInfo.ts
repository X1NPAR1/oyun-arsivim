import { invoke } from "@tauri-apps/api/core";

/** Rust tarafındaki `app_info` komutunun döndürdüğü yapı. */
export interface AppInfo {
  name: string;
  version: string;
}

export function getAppInfo(): Promise<AppInfo> {
  return invoke<AppInfo>("app_info");
}
