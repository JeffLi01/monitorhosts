use std::{collections::BTreeMap, sync::{Arc, RwLock}};

use log::trace;
use slint::*;

use crate::{
    manager::{HostConfig, Manager, Port}, ui::AddDialog
};

pub fn add_dialog(mgr: Arc<RwLock<Manager>>) -> AddDialog {
    let dialog = AddDialog::new().unwrap();
    let dialog_weak = dialog.as_weak();
    let dialog_clone = dialog_weak.clone();
    let mgr = mgr.clone();
    dialog.on_action_ok(move |host| {
        trace!("add-dialog::on_action_ok: {host:?}");
        let name = host.name.to_string();
        let mut ports = BTreeMap::new();
        ports.insert(Port::Http, host.http);
        ports.insert(Port::Https, host.https);
        ports.insert(Port::Ssh, host.ssh);
        ports.insert(Port::Vnc, host.vnc);
        ports.insert(Port::Ipmi, host.ipmi);
        trace!("calling hmanager::insert_host...");
        mgr.write().unwrap().insert_host(HostConfig::new(name, ports));
        trace!("calling hmanager::insert_host done");
        dialog_clone.unwrap().hide().unwrap();
    });
    let dialog_clone = dialog_weak.clone();
    dialog.on_action_cancel(move || {
        trace!("add-dialog::on_action_cancel");
        dialog_clone.unwrap().hide().unwrap();
    });
    dialog
}
