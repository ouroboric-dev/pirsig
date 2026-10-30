//! Pirsig on the desktop (macOS); Android starts from `lib.rs` instead.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    pirsig_desktop_lib::run();
}
