#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod runner;
mod gnats;

fn main() {
    let effect = gnats::Gnats::new();
    runner::run_main(effect, "gnats");
}
