use std::sync::mpsc;
use std::thread;

use log::{trace, warn};
use slint::ComponentHandle;
use tray_item::{IconSource, TrayItem};

use crate::ui::MainWindow;

pub struct Tray {
    pub thread: std::thread::JoinHandle<()>,
}

impl Tray {
    pub fn new(window: &MainWindow) -> Self {
        let window_weak = window.as_weak();

        let thread = thread::spawn(move || {
            let mut tray =
                TrayItem::new("MonitorHosts", IconSource::Resource("default-icon")).unwrap();
            tray.inner_mut().set_tooltip("MonitorHosts").unwrap();

            tray.add_label("MonitorHosts").unwrap();

            tray.inner_mut().add_separator().unwrap();

            let (tx, rx) = mpsc::sync_channel(1);

            let tx_clone = tx.clone();
            tray.add_menu_item("主界面", move || {
                tx_clone.send(Message::ShowMainWindow).unwrap();
            })
            .unwrap();

            // let tx_clone = tx.clone();
            // tray.add_menu_item("配置", move || {
            //     tx_clone.send(Message::Config).unwrap();
            // })
            // .unwrap();

            // let tx_clone = tx.clone();
            // tray.add_menu_item("关于", move || {
            //     tx_clone.send(Message::About).unwrap();
            // })
            // .unwrap();

            tray.inner_mut().add_separator().unwrap();

            let tx_clone = tx.clone();
            tray.add_menu_item("退出", move || {
                tx_clone.send(Message::Quit).unwrap();
            })
            .unwrap();

            loop {
                match rx.recv() {
                    Ok(Message::Quit) => {
                        warn!("terminating...");
                        slint::quit_event_loop().unwrap();
                        break;
                    }
                    Ok(Message::ShowMainWindow) => {
                        trace!("show main window...");
                        window_weak
                            .upgrade_in_event_loop(|window| {
                                trace!("before showing main window");
                                window.show().unwrap();
                                trace!("after showing main window");
                            })
                            .unwrap();
                    }
                    // Ok(Message::Config) => {
                    //     println!("Config");
                    // }
                    // Ok(Message::About) => {
                    //     println!("About");
                    // }
                    Err(err) => eprintln!("{}", err),
                }
            }
        });
        Self { thread }
    }

    pub fn join(self) {
        trace!("wating tray to terminate...");
        self.thread.join().unwrap();
        trace!("wating tray to terminate...done");
    }
}

pub enum Message {
    // About,
    // Config,
    ShowMainWindow,
    Quit,
}
