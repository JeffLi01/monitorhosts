use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use global_hotkey::HotKeyState;
use global_hotkey::{hotkey::{Code, HotKey, Modifiers}, GlobalHotKeyEvent, GlobalHotKeyManager};
use log::{info, trace};

use crate::manager::{HostConfig, Manager};

#[allow(dead_code)]
pub struct HotkeyWorker {
    terminate_flag: Arc<AtomicBool>,
    thread: std::thread::JoinHandle<()>,
    manager: GlobalHotKeyManager,
}

impl HotkeyWorker {
    pub fn new(manager: Arc<RwLock<Manager>>) -> Self {
        let mut clipboard = arboard::Clipboard::new().unwrap();
        let terminate_flag = Arc::new(AtomicBool::new(false));
        // initialize the hotkeys manager
        let hotkey_manager = GlobalHotKeyManager::new().unwrap();
        
        // construct the hotkey
        let modifiers = Modifiers::CONTROL | Modifiers::ALT | Modifiers::SHIFT;
        let hotkey = HotKey::new(Some(modifiers), Code::KeyM);
        info!("register hotkey: {hotkey:?}");
        hotkey_manager.register(hotkey).expect("hotkey {hotkey:?} should not be occupied");

        let hotkey_channel = GlobalHotKeyEvent::receiver();

        let flag = terminate_flag.clone();

        let thread = thread::spawn(move || {
            loop {
                if flag.load(Ordering::Relaxed) {
                    return;
                }
                match hotkey_channel.recv_timeout(Duration::from_secs(1)) {
                    Ok(event) => {
                        trace!("hotkey({}) {:?}", event.id, event.state);
                        match event.state {
                            HotKeyState::Pressed => {
                                let name = clipboard.get_text().unwrap();
                                trace!("Clipboard text was: {name}");
                                let host = HostConfig::with_all_enable(name);
                                manager.write().unwrap().add_host(host);
                            },
                            HotKeyState::Released => {},
                        }
                    },
                    Err(_) => {},
                };
            };
        });
        Self {
            terminate_flag,
            thread,
            manager: hotkey_manager,
        }
    }

    pub fn join(self) {
        trace!("wating hotkey worker to terminate...");
        self.terminate_flag.store(true, Ordering::Relaxed);
        self.thread.join().unwrap();
        trace!("wating hotkey worker to terminate...done");
    }
}
