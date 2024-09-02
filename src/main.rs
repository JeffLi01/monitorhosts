pub mod ui {
    slint::include_modules!();
}

mod tray;

use std::sync::{atomic::{AtomicBool, Ordering}, Arc};

use tray::Message;
use ui::*;

fn main() {
    let shown_flag = Arc::new(AtomicBool::new(false));
    let window = MainWindow::new().unwrap();
    let (join_handle, rx) = tray::setup(shown_flag.clone());
    loop {
        match rx.recv() {
            Ok(Message::Quit) => {
                println!("Quit");
                break;
            }
            Ok(Message::ShowMainWindow) => {
                println!("MainWindow");
                shown_flag.store(true, Ordering::Relaxed);
                window.run().unwrap();
                shown_flag.store(false, Ordering::Relaxed);
            }
            Ok(Message::Add) => {
                println!("Add");
            }
            Ok(Message::Clear) => {
                println!("Clear");
            }
            Ok(Message::Config) => {
                println!("Config");
            }
            Ok(Message::About) => {
                println!("About");
            }
            Err(err) => eprintln!("{}", err),
        }
    }
    join_handle.join().unwrap();
}
