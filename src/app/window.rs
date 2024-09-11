use std::sync::{Arc, RwLock};

use slint::*;

use crate::{manager::Manager, ui::MainWindow};

pub fn setup(mgr: Arc<RwLock<Manager>>) -> MainWindow {
    let window = MainWindow::new().unwrap();

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
