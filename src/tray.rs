use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;
use std::thread;

use tray_item::{IconSource, TrayItem};

pub struct Tray {
    pub msg_channel: Receiver<Message>,
    pub thread: std::thread::JoinHandle<()>,
}

impl Tray {
    pub fn new(shown_flag: Arc<AtomicBool>) -> Self {
        let (tx, msg_channel) = mpsc::sync_channel(1);

        let thread = thread::spawn(move || {
            let mut tray =
                TrayItem::new("MonitorHosts", IconSource::Resource("default-icon")).unwrap();
            tray.inner_mut().set_tooltip("MonitorHosts").unwrap();

            tray.add_label("MonitorHosts").unwrap();

            tray.inner_mut().add_separator().unwrap();

            let (tx_inner, rx_inner) = mpsc::sync_channel(1);

            let tx_clone = tx_inner.clone();
            tray.add_menu_item("主界面", move || {
                tx_clone.send(Message::ShowMainWindow).unwrap();
            })
            .unwrap();

            let tx_clone = tx_inner.clone();
            tray.add_menu_item("配置", move || {
                tx_clone.send(Message::Config).unwrap();
            })
            .unwrap();

            let tx_clone = tx_inner.clone();
            tray.add_menu_item("关于", move || {
                tx_clone.send(Message::About).unwrap();
            })
            .unwrap();

            tray.inner_mut().add_separator().unwrap();

            let tx_clone = tx_inner.clone();
            tray.add_menu_item("退出", move || {
                tx_clone.send(Message::Quit).unwrap();
            })
            .unwrap();

            loop {
                match rx_inner.recv() {
                    Ok(Message::Quit) => {
                        println!("Quit");
                        let shown = shown_flag.load(Ordering::Relaxed);
                        println!("shown: {}", shown);
                        tx.send(Message::Quit).unwrap();
                        break;
                    }
                    Ok(Message::ShowMainWindow) => {
                        println!("MainWindow");
                        let shown = shown_flag.load(Ordering::Relaxed);
                        println!("shown: {}", shown);
                        if !shown {
                            tx.send(Message::ShowMainWindow).unwrap();
                        }
                    }
                    Ok(msg) => {
                        tx.send(msg).unwrap();
                    }
                    Err(err) => eprintln!("{}", err),
                }
            }
        });
        Self {
            msg_channel,
            thread,
        }
    }

    pub fn join(self) {
        self.thread.join().unwrap();
    }
}

pub enum Message {
    About,
    Config,
    ShowMainWindow,
    Quit,
}
