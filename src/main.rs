#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod gnats;

fn main() {
    let effect = gnats::Gnats::new();
    library::screensaver_runner::run_main(effect, "gnats");
}
