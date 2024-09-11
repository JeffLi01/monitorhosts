use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use global_hotkey::HotKeyState;
use global_hotkey::{hotkey::{Code, HotKey, Modifiers}, GlobalHotKeyEvent, GlobalHotKeyManager};
use log::{error, info, trace, warn};
use notify_rust::Notification;
use url::Url;

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
                                let text = clipboard.get_text().unwrap();
                                trace!("Clipboard text was: {text}");
                                let name = match extract(&text) {
                                    Some(name) => name,
                                    None => {
                                        warn!("extract: no valid ip found in {text}");
                                        continue;
                                    },
                                };
                                let host = HostConfig::with_all_enable(name.clone());
                                manager.write().unwrap().add_host(host);
                                if let Err(err) = Notification::new()
                                    .summary("MonitorHosts")
                                    .body(&format!("添加主机：'{name}'"))
                                    .show() {
                                        error!("failed to show notification for adding host {name}: {err}");
                                    }
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

fn extract<S: std::fmt::Display>(text: S) -> Option<String> {
    let name = format!("{text}");
    let u = match Url::parse(&name) {
        Ok(u) => u,
        Err(err) => {
            error!("failed to parse url '{name}': {err}");
            return None;
        },
    };
    u.host_str().map(ToOwned::to_owned)
}