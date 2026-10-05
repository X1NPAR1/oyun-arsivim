// Release build'de ek konsol penceresi açılmasını engeller.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    oyun_arsivim_lib::run()
}
