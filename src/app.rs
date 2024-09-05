use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, RwLock,
};

use slint::*;

mod dialog;
mod window;

use crate::{controllers::monitor::Monitor, manager::Port};
use crate::{
    manager::Manager,
    tray::{Message, Tray},
    ui::MainWindow,
};

const PORTS: [Port; 5] = [Port::Http, Port::Https, Port::Ssh, Port::Vnc, Port::Ipmi];

pub struct Application {
    pub shown: Arc<AtomicBool>,
    pub window: MainWindow,
    tray: Tray,
    manager: Arc<RwLock<Manager>>,
}

impl Application {
    pub fn new() -> Self {
        let manager = Arc::new(RwLock::new(Manager::new()));
        let shown = Arc::new(AtomicBool::new(false));
        let window = window::setup(manager.clone());
        let tray = Tray::new(shown.clone());

        Application {
            manager,
            shown,
            window,
            tray,
        }
    }

    pub fn run(self) {
        let monitor = Monitor::new(self.manager.clone(), &self.window);
        loop {
            match self.tray.msg_channel.recv() {
                Ok(Message::Quit) => {
                    break;
                }
                Ok(Message::ShowMainWindow) => {
                    self.shown.store(true, Ordering::Relaxed);
                    self.window.run().unwrap();
                    self.shown.store(false, Ordering::Relaxed);
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
        self.tray.join();
        monitor.join();
    }
}
