#![windows_subsystem = "windows"]

#[path = "gui/gui.rs"]
mod gui;

fn main() {
    if let Err(err) = gui::run() {
        eprintln!("{err:#}");
        std::process::exit(1);
    }
}
