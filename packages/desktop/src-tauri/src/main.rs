// main.rs - Application entry point

// Prevents additional console window on Windows in release mode
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    focusflow_lib::run();
}
