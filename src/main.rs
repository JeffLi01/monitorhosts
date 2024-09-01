pub mod ui {
    slint::include_modules!();
}

mod tray;

use tray::Message;
use ui::*;

fn main() {
    let (join_handle, rx) = tray::setup();
    loop {
        match rx.recv() {
            Ok(Message::Quit) => {
                println!("Quit");
                break;
            }
            Ok(Message::MainWindow) => {
                println!("MainWindow");
                let window = MainWindow::new().unwrap();
                window.run().unwrap();
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
