use std::sync::mpsc;
use tray_item::{IconSource, TrayItem};

enum Message {
    Quit,
    Add,
    Clear,
    Config,
    About,
}

fn main() {
    let mut tray = TrayItem::new(
        "MonitorHosts",
        IconSource::Resource("default-icon"),
    )
    .unwrap();
    tray.inner_mut().set_tooltip("MonitorHosts").unwrap();

    tray.add_label("MonitorHosts").unwrap();

    tray.inner_mut().add_separator().unwrap();

    let (tx, rx) = mpsc::sync_channel(1);

    let tx_clone = tx.clone();
    tray.add_menu_item("添加", move || {
        tx_clone.send(Message::Add).unwrap();
    })
    .unwrap();

    let tx_clone = tx.clone();
    tray.add_menu_item("清除", move || {
        tx_clone.send(Message::Clear).unwrap();
    })
    .unwrap();

    let tx_clone = tx.clone();
    tray.add_menu_item("配置", move || {
        tx_clone.send(Message::Config).unwrap();
    })
    .unwrap();

    let tx_clone = tx.clone();
    tray.add_menu_item("关于", move || {
        tx_clone.send(Message::About).unwrap();
    })
    .unwrap();

    tray.inner_mut().add_separator().unwrap();

    let tx_clone = tx.clone();
    tray.add_menu_item("Quit", move || {
        tx_clone.send(Message::Quit).unwrap();
    })
    .unwrap();

    loop {
        match rx.recv() {
            Ok(Message::Quit) => {
                println!("Quit");
                break;
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
            _ => {}
        }
    }
}
