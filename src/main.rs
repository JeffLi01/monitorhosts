#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod ui {
    slint::include_modules!();
}

mod app;
mod args;
mod controllers {
    pub mod monitor;
}
mod hotkey;
mod logging;
mod manager;
mod tray;

use clap::Parser;

use app::Application;
use args::Args;

fn main() {
    let args = Args::parse();
    logging::setup(args.log_level());

    let app = Application::new();
    app.run();
}
