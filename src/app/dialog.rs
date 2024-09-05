use std::{collections::BTreeMap, sync::{Arc, RwLock}};

use log::{trace, warn};
use slint::*;

use crate::{
    manager::{HostConfig, Manager, Port}, ui::{ConfirmDialog, HostConfigDialog}
};
use crate::ui::HostConfigModel;

pub fn add_dialog(mgr: Arc<RwLock<Manager>>) -> HostConfigDialog {
    let dialog = HostConfigDialog::new().unwrap();
    let dialog_weak = dialog.as_weak();
    let dialog_clone = dialog_weak.clone();
    let mgr = mgr.clone();
    dialog.on_action_ok(move |host| {
        trace!("add-dialog::on_action_ok: {host:?}");
        let name = host.name.to_string();
        if !mgr.read().unwrap().contains_host(&name) {
            let mut ports = BTreeMap::new();
            ports.insert(Port::Http, host.http);
            ports.insert(Port::Https, host.https);
            ports.insert(Port::Ssh, host.ssh);
            ports.insert(Port::Vnc, host.vnc);
            ports.insert(Port::Ipmi, host.ipmi);
            trace!("calling hmanager::add_host...");
            mgr.write().unwrap().add_host(HostConfig::new(name, ports));
            trace!("calling hmanager::add_host done");
        } else {
            warn!("host with name {name} already exists");
        }
        dialog_clone.unwrap().hide().unwrap();
    });
    let dialog_clone = dialog_weak.clone();
    dialog.on_action_cancel(move || {
        trace!("add-dialog::on_action_cancel");
        dialog_clone.unwrap().hide().unwrap();
    });
    dialog
}

pub fn config_dialog(mgr: Arc<RwLock<Manager>>, index: usize) -> HostConfigDialog {
    let m = mgr.read().unwrap();
    let host = m.get_host(index).expect("the index {index} should be valid");
    let dialog = HostConfigDialog::new().unwrap();
    dialog.set_dialog_title("配置".into());
    let mut model = HostConfigModel::default();
    model.name = SharedString::from(&host.name);
    host.ports
        .iter()
        .for_each(|(port, enabled)| {
            match port {
                Port::Http => model.http = *enabled,
                Port::Https => model.https = *enabled,
                Port::Ssh => model.ssh = *enabled,
                Port::Vnc => model.vnc = *enabled,
                Port::Ipmi => model.ipmi = *enabled,
            }
        });
    dialog.set_input_model(model);
    let dialog_weak = dialog.as_weak();
    let dialog_clone = dialog_weak.clone();
    let mgr = mgr.clone();
    dialog.on_action_ok(move |host| {
        trace!("config-dialog::on_action_ok: {host:?}");
        let name = host.name.to_string();
        if mgr.read().unwrap().contains_host(&name) {
            let mut ports = BTreeMap::new();
            ports.insert(Port::Http, host.http);
            ports.insert(Port::Https, host.https);
            ports.insert(Port::Ssh, host.ssh);
            ports.insert(Port::Vnc, host.vnc);
            ports.insert(Port::Ipmi, host.ipmi);
            trace!("calling hmanager::update_host...");
            mgr.write().unwrap().update_host(index, HostConfig::new(name, ports));
            trace!("calling hmanager::update_host done");
        } else {
            warn!("host with name {name} does not exist");
        }
        dialog_clone.unwrap().hide().unwrap();
    });
    let dialog_clone = dialog_weak.clone();
    dialog.on_action_cancel(move || {
        trace!("config-dialog::on_action_cancel");
        dialog_clone.unwrap().hide().unwrap();
    });
    dialog
}

pub fn remove_dialog(mgr: Arc<RwLock<Manager>>, index: usize) -> ConfirmDialog {
    let m = mgr.read().unwrap();
    let host = m.get_host(index).expect("the index {index} should be valid");
    let dialog = ConfirmDialog::new().unwrap();
    dialog.set_dialog_title("删除".into());
    dialog.set_confirm_message(slint::format!("确定要删除 '{}'?", host.name));
    let dialog_weak = dialog.as_weak();
    let dialog_clone = dialog_weak.clone();
    let mgr = mgr.clone();
    dialog.on_action_ok(move || {
        trace!("remove-dialog::on_action_ok");
        mgr.write().unwrap().remove_host(index);
        dialog_clone.unwrap().hide().unwrap();
    });
    let dialog_clone = dialog_weak.clone();
    dialog.on_action_cancel(move || {
        trace!("remove-dialog::on_action_cancel");
        dialog_clone.unwrap().hide().unwrap();
    });
    dialog
}

pub fn clear_dialog(mgr: Arc<RwLock<Manager>>) -> ConfirmDialog {
    let dialog = ConfirmDialog::new().unwrap();
    dialog.set_dialog_title("清除".into());
    dialog.set_confirm_message("确定要清楚所有主机么？".into());
    let dialog_weak = dialog.as_weak();
    let dialog_clone = dialog_weak.clone();
    let mgr = mgr.clone();
    dialog.on_action_ok(move || {
        trace!("clear-dialog::on_action_ok");
        mgr.write().unwrap().clear_host();
        dialog_clone.unwrap().hide().unwrap();
    });
    let dialog_clone = dialog_weak.clone();
    dialog.on_action_cancel(move || {
        trace!("clear-dialog::on_action_cancel");
        dialog_clone.unwrap().hide().unwrap();
    });
    dialog
}
