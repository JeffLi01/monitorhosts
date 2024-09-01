use std::sync::mpsc::{self, Receiver};
use std::thread;

use tray_item::{IconSource, TrayItem};

pub enum Message {
    About,
    Add,
    Clear,
    Config,
    MainWindow,
    Quit,
}

pub fn setup() -> (std::thread::JoinHandle<()>, Receiver<Message>) {
    let (tx, rx) = mpsc::sync_channel(1);

    let handle = thread::spawn(move || {
        let mut tray = TrayItem::new(
            "MonitorHosts",
            IconSource::Resource("default-icon"),
        )
        .unwrap();
        tray.inner_mut().set_tooltip("MonitorHosts").unwrap();

        tray.add_label("MonitorHosts").unwrap();

        tray.inner_mut().add_separator().unwrap();

        let (tx_inner, rx_inner) = mpsc::sync_channel(1);

        let tx_clone = tx_inner.clone();
        tray.add_menu_item("主界面", move || {
            tx_clone.send(Message::MainWindow).unwrap();
        })
        .unwrap();

        let tx_clone = tx_inner.clone();
        tray.add_menu_item("添加", move || {
            tx_clone.send(Message::Add).unwrap();
        })
        .unwrap();

        let tx_clone = tx_inner.clone();
        tray.add_menu_item("清除", move || {
            tx_clone.send(Message::Clear).unwrap();
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
        tray.add_menu_item("Quit", move || {
            tx_clone.send(Message::Quit).unwrap();
        })
        .unwrap();

        loop {
            match rx_inner.recv() {
                Ok(Message::Quit) => {
                    tx.send(Message::Quit).unwrap();
                    break;
                }
                Ok(msg) => {
                    tx.send(msg).unwrap();
                }
                Err(err) => eprintln!("{}", err),
            }
        }
    });
    (handle, rx)
}
