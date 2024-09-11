use std::path::PathBuf;
use std::sync::{Arc, RwLock};

mod dialog;
mod window;

use crate::{controllers::monitor::Monitor, hotkey::HotkeyWorker};
use crate::{manager::Manager, tray::Tray, ui::MainWindow};

pub struct Application {
    config: PathBuf,
    pub window: MainWindow,
    tray: Tray,
    manager: Arc<RwLock<Manager>>,
    hotkey: HotkeyWorker,
}

impl Application {
    pub fn new() -> Self {
        let s = dirs::config_dir()
            .expect("config_dir should be valid")
            .to_str()
            .expect("config_dir should contains only UTF-8")
            .to_owned();
        let mut config = PathBuf::from(&s);
        config.push("monitorhosts.json");
        let mgr = Manager::with_config(&config).unwrap_or_else(Manager::new);
        let manager = Arc::new(RwLock::new(mgr));
        let window = window::setup(manager.clone());
        let tray = Tray::new(&window);
        let hotkey = HotkeyWorker::new(manager.clone());

        Application {
            config,
            manager,
            window,
            tray,
            hotkey,
        }
    }

    pub fn run(self) {
        let monitor = Monitor::new(self.manager.clone(), &self.window);
        slint::run_event_loop_until_quit().unwrap();
        self.tray.join();
        monitor.join();
        self.hotkey.join();

        let hosts = self.manager.read().unwrap().hosts.clone();
        let toml = serde_json::to_string(&hosts).unwrap();
        std::fs::write(&self.config, &toml).expect("config '{config:?}' should be writable");
    }
}
