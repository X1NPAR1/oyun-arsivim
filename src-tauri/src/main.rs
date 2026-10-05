// Release build'de ek konsol penceresi açılmasını engeller.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // UAC onaylı yardımcı görev kipi: arayüz açılmaz, yalnız beyaz listedeki ayar yazılır.
    if let Some(code) = oyun_arsivim_lib::handle_admin_task() {
        std::process::exit(code);
    }
    oyun_arsivim_lib::run()
}
