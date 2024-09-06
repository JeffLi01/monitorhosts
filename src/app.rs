use std::sync::{Arc, RwLock};

mod dialog;
mod window;

use crate::{controllers::monitor::Monitor, manager::Port};
use crate::{
    manager::Manager,
    tray::Tray,
    ui::MainWindow,
};

const PORTS: [Port; 5] = [Port::Http, Port::Https, Port::Ssh, Port::Vnc, Port::Ipmi];

pub struct Application {
    pub window: MainWindow,
    tray: Tray,
    manager: Arc<RwLock<Manager>>,
}

impl Application {
    pub fn new() -> Self {
        let manager = Arc::new(RwLock::new(Manager::new()));
        let window = window::setup(manager.clone());
        let tray = Tray::new(&window);

        Application {
            manager,
            window,
            tray,
        }
    }

    pub fn run(self) {
        let monitor = Monitor::new(self.manager.clone(), &self.window);
        slint::run_event_loop_until_quit().unwrap();
        self.tray.join();
        monitor.join();
    }
}
