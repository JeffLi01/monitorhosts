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

    window.on_add_host(move |id| {
        println!("{id}");
        let dialog = super::dialog::setup(mgr.clone());
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