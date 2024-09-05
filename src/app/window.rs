use std::sync::{Arc, RwLock};

use slint::*;

use crate::{
    manager::Manager,
    ui::{MainWindow, MainWindowAdapter},
};

use super::PORTS;

pub fn setup(mgr: Arc<RwLock<Manager>>) -> MainWindow {
    let window = MainWindow::new().unwrap();

    let titles: Vec<SharedString> = get_titles()
        .iter()
        .map(|s| SharedString::from(s))
        .collect();
    window.global::<MainWindowAdapter>().set_titles(ModelRc::new(VecModel::from(titles)));

    let manager = mgr.clone();
    window.on_add_host(move || {
        let dialog = super::dialog::add_dialog(manager.clone());
        dialog.show().unwrap();
    });

    let manager = mgr.clone();
    window.on_config_host(move |index| {
        if index >= 0 {
            let dialog = super::dialog::config_dialog(manager.clone(), index as usize);
            dialog.show().unwrap();
        }
    });

    let manager = mgr.clone();
    window.on_remove_host(move |index| {
        if index >= 0 {
            let dialog = super::dialog::remove_dialog(manager.clone(), index as usize);
            dialog.show().unwrap();
        }
    });

    let manager = mgr.clone();
    window.on_clear_hosts(move || {
        let dialog = super::dialog::clear_dialog(manager.clone());
        dialog.show().unwrap();
    });

    window
}

fn get_titles() -> Vec<String> {
    let mut titles = vec!["主机名".to_string()];
    PORTS.iter()
        .for_each(|port| titles.push(port.name()));
    titles
}